use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::JoinHandle,
};

use crate::extractors::ExtractedTags;
use crate::init_interpreter;
use anyhow::Result;
use anyhow::anyhow;
use rustpython_vm::compiler;
use serde_json::Value;

struct TaggerWorkerJob {
    code: String,
    json_input: String,
    when_done: Sender<Result<Option<ExtractedTags>>>,
}

pub struct TaggerWorker {
    thread_handle: Option<JoinHandle<()>>,
    job_tx: Option<Sender<TaggerWorkerJob>>,
}

impl Drop for TaggerWorker {
    fn drop(&mut self) {
        self.job_tx.take();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl TaggerWorker {
    pub fn new() -> Result<Self> {
        let (job_tx, job_rx): (Sender<TaggerWorkerJob>, Receiver<TaggerWorkerJob>) =
            mpsc::channel();

        let thread = std::thread::Builder::new()
            .name("rustpython_tagger_worker".to_owned())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let interpreter = init_interpreter();

                while let Ok(job) = job_rx.recv() {
                    let result = interpreter.enter(|vm| {
                        let scope = vm.new_scope_with_builtins();

                        let compiled = vm
                            .compile(&job.code, compiler::Mode::Exec, "<embedded>".to_string())
                            .map_err(|err| anyhow!("Compile error: {:?}", err))?;

                        vm.run_code_obj(compiled, scope.clone())
                            .map_err(|err| anyhow!("Runtime error: {:?}", err))?;

                        let parser_function = scope
                            .globals
                            .get_item("extract", vm)
                            .map_err(|_| anyhow!("Function 'extract' not found"))?;

                        let json_module = vm
                            .import("json", 0)
                            .map_err(|err| anyhow!("Failed to import json: {:?}", err))?;

                        let loads_function = json_module
                            .get_attr("loads", vm)
                            .map_err(|err| anyhow!("json.loads not found: {:?}", err))?;

                        let input = loads_function
                            .call((vm.ctx.new_str(job.json_input.as_str()),), vm)
                            .map_err(|err| anyhow!("Input deserialization error: {:?}", err))?;

                        let result = parser_function
                            .call((input,), vm)
                            .map_err(|err| anyhow!("Execution error: {:?}", err))?;

                        let dumps_function = json_module
                            .get_attr("dumps", vm)
                            .map_err(|err| anyhow!("json.dumps not found: {:?}", err))?;

                        let json_result = dumps_function
                            .call((result,), vm)
                            .map_err(|err| anyhow!("Serialization error: {:?}", err))?;

                        let output = json_result
                            .str(vm)
                            .map_err(|_| anyhow!("Output string conversion error"))?
                            .to_string();

                        let serialized: ExtractedTags = serde_json::from_str(&output)?;

                        Ok::<_, anyhow::Error>(Some(serialized))
                    });

                    let _ = job.when_done.send(result);
                }
            })?;

        Ok(Self {
            thread_handle: Some(thread),
            job_tx: Some(job_tx),
        })
    }

    pub fn push_job(
        &self,
        code: &str,
        json_input: &Value,
    ) -> Result<Receiver<Result<Option<ExtractedTags>>>> {
        let (done_tx, done_rx) = mpsc::channel();

        if let Some(tx) = &self.job_tx {
            let job = TaggerWorkerJob {
                code: code.to_string(),
                json_input: serde_json::to_string(json_input)?,
                when_done: done_tx,
            };
            tx.send(job)?;

            Ok(done_rx)
        } else {
            Err(anyhow::anyhow!(
                "trying to push a job to a worker thread that already shut down"
            ))
        }
    }
}
