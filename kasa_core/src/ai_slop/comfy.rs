use std::{
    collections::HashSet,
    fmt::Debug,
    path::PathBuf,
};

use anyhow::Result;
use indexmap::IndexMap;
use itertools::Itertools;
use log::{error, trace};
use serde::{Deserialize, Serialize};


use super::{
    errors::SlopTagParseError,
    prompt_parser::parse_prompt,
    supported_formats::{parse_png_meta, SLOP_SUPPORTED_FORMATS},
    SlopTag, SlopTags,
};

/// input is the `Prompt` field of meta
pub fn parse_comfy_tags_from_meta(input: &ComfyPrompt) -> SlopTags {
    //let parsed: ComfyPrompt = serde_json::from_str(input).unwrap();

    let mut initial_positive_node_ids: Vec<&str> = vec![];
    let mut initial_negative_node_ids: Vec<&str> = vec![];

    for node in &input.nodes {
        if let Some(positive_nodes) = &node.1.inputs.positive {
            for positive_node in positive_nodes {
                if let PromptVal::Val(positive_node_id) = positive_node {
                    initial_positive_node_ids.push(&positive_node_id);
                }
            }
        }

        if let Some(negative_nodes) = &node.1.inputs.negative {
            for negative_node in negative_nodes {
                if let PromptVal::Val(negative_node_id) = negative_node {
                    initial_negative_node_ids.push(&negative_node_id);
                }
            }
        }
    }

    let mut positive_node_ids: Vec<String> = vec![];
    let mut negative_node_ids: Vec<String> = vec![];

    // positive nodes
    for node_id in &initial_positive_node_ids {
        // does this need to be a string?

        let _final = get_final_node(input, node_id);

        if let Some(final_node_ref) = _final {
            positive_node_ids.push(final_node_ref);
        }
    }

    // negative nodes
    for node_id in &initial_negative_node_ids {
        // does this need to be a string?

        let _final = get_final_node(input, node_id);

        if let Some(final_node_ref) = _final {
            negative_node_ids.push(final_node_ref);
        }
    }

    let mut positive_prompts: Vec<String> = vec![]; //TODO Vec<String> -> Vec<&str>
    let mut negative_prompts: Vec<String> = vec![];

    // positive prompts
    for node_id in positive_node_ids {
        if let Some(node) = input.nodes.get(&node_id) {
            if let Some(text) = &node.inputs.text {
                let v = match text {
                    CuresedTextVal::Single(prompt_val) => prompt_val,
                    CuresedTextVal::Vec(vec) => vec.get(0).unwrap_or(&PromptVal::None),
                };

                if let PromptVal::Val(text) = v {
                    positive_prompts.push(text.to_owned());
                }
            }

            if let Some(text) = &node.inputs.result {
                if let PromptVal::Val(text) = text {
                    positive_prompts.push(text.to_owned());
                }

                // For some reason the extension's text merge node doesn't have the result field on negative
                // prompts, i know this is the positive prompt but this is here "just in case"
            }
            if let Some(text) = &node.inputs.text_a {
                if let PromptVal::Val(text) = text {
                    positive_prompts.push(text.to_owned());
                }
            }
            if let Some(text) = &node.inputs.text_b {
                if let PromptVal::Val(text) = text {
                    positive_prompts.push(text.to_owned());
                }
            }
        }
    }

    // negative prompts
    for node_id in negative_node_ids {
        if let Some(node) = input.nodes.get(&node_id) {
            if let Some(text) = &node.inputs.text {
                let v = match text {
                    CuresedTextVal::Single(prompt_val) => prompt_val,
                    CuresedTextVal::Vec(vec) => vec.get(0).unwrap_or(&PromptVal::None),
                };

                if let PromptVal::Val(text) = v {
                    negative_prompts.push(text.to_owned());
                }
            }

            // Used by some comfy extension
            if let Some(text) = &node.inputs.result {
                if let PromptVal::Val(text) = text {
                    negative_prompts.push(text.to_owned());
                }

                // For some reason the extension's text merge node doesn't have the result field on negative prompts
                // further "deduplication" on next step should merge them properly... i hope
            }

            if let Some(text) = &node.inputs.text_a {
                if let PromptVal::Val(text) = text {
                    negative_prompts.push(text.to_owned());
                }
            }

            if let Some(text) = &node.inputs.text_b {
                if let PromptVal::Val(text) = text {
                    negative_prompts.push(text.to_owned());
                }
            }
        }
    }

    let mut positive_tags: HashSet<SlopTag> = HashSet::new();
    let mut negative_tags: HashSet<SlopTag> = HashSet::new();

    for prompt in positive_prompts {
        let tags = parse_prompt(&prompt);
        let tags_hm: HashSet<SlopTag> = HashSet::from_iter(tags.iter().cloned());
        positive_tags.extend(tags_hm);
    }

    for prompt in negative_prompts {
        let tags = parse_prompt(&prompt);
        let tags_hm: HashSet<SlopTag> = HashSet::from_iter(tags.iter().cloned());
        negative_tags.extend(tags_hm);
    }

    SlopTags {
        positive: positive_tags.into_iter().collect(),
        negative: negative_tags.into_iter().collect(),
    }
}

fn get_final_node(input: &ComfyPrompt, node_id: &str) -> Option<String> {
    let mut max_loop = 1000;
    let mut current_node_id = node_id.to_string();

    let _final: Option<String> = loop {
        if max_loop == 0 {
            error!("ComfyUI Prompt Parser: ran into a recursive node, what kind of spaghetti nodes does that image contain");
            break None; // recursive nodes? is that even possible
        }

        let Some(current_node) = &input.nodes.get(&current_node_id) else {
            error!("ComfyUI Prompt Parser: ran into a node reference that doesn't exist.");
            break None; // that node does not exists, return None
        };

        // for some reason positive prompts have a result field with both prompts
        // combined but the negative prompts doesn't wtf is wrong with this extension
        // now i need to implement text merging
        if current_node.inputs.result.is_some()
            || current_node.inputs.text_a.is_some()
            || current_node.inputs.text_b.is_some()
        {
            trace!("ComfyUI Prompt Parser: parsing text merge nodes.");
            break Some(current_node_id);
        }

        let Some(text) = &current_node.inputs.text else {
            error!("ComfyUI Prompt Parser: The referenced positive/negative node doesn't have a text field on it");
            break None; // this node doesn't have a text field, just return
        };

        let text_val = match text {
            CuresedTextVal::Single(prompt_val) => prompt_val,
            CuresedTextVal::Vec(vec) => {
                if let Some(v) = vec.get(0) {
                    v
                } else {
                    break None;
                }
            }
        };

        // let PromptVal::Val(val) = text_val {};
        // doesn't work, back to spaghetti nested if let's
        if let PromptVal::Val(val) = text_val {
            // check if it is an number to see if it is an reference
            // i don't know if there is a better way, no i am not reading comfy's source code
            // i am too lazy for that
            let Some(next_node_id) = val.parse::<i32>().ok() else {
                break Some(current_node_id);
            };

            current_node_id = next_node_id.to_string();
            max_loop -= 1;
        } else {
            error!("ComfyUI Prompt Parser: The referenced nodes text fields first index only has an integer");
            break None; // the value is a int?, what does that even mean?
        }
    };

    return _final;
}

#[deprecated]
fn parse_comfy_tags(path: PathBuf) -> Result<SlopTags> {
    let mime = mime_guess::from_path(&path).first();

    let mime = match mime {
        Some(mime) => {
            if !SLOP_SUPPORTED_FORMATS.contains(&mime.to_string().as_ref()) {
                return Err(SlopTagParseError::UnsupportedFile(mime.to_string()).into());
            }
            mime
        }
        None => return Err(SlopTagParseError::UnsupportedFile("Unknown".to_string()).into()),
    };

    match mime.to_string().as_ref() {
        "image/png" => {
            let meta = parse_png_meta(&path)?;

            match meta.get("prompt") {
                Some(prompt) => unimplemented!(), //return Ok(parse_comfy_tags_from_meta(&prompt)),
                None => {
                    unreachable!("Presence of 'prompt' png field should be checked while determining if the AI metadata is present")
                }
            }
        }

        // Other supported image formats that use UserComment
        _ => {
            todo!()
        }
    }

    todo!()
}

#[derive(Debug, Serialize, Deserialize)]
/// Minimal struct containing only the fields required to get prompts
pub struct ComfyPrompt {
    #[serde(flatten)]
    // std::HashMap apparently doesn't keep order, serde_json has an option for the serde_json::Map to
    // keep order but it doesn't support Serialize, Deserialize, Debug traits for some reason
    nodes: IndexMap<String, ComfyNode>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComfyNode {
    inputs: ComfyNodeInputs,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComfyNodeInputs {
    text: Option<CuresedTextVal>,

    /// Used by some comfy extension to merge prompts
    // I am not sure about the types though
    result: Option<PromptVal>,
    text_a: Option<PromptVal>,
    text_b: Option<PromptVal>,

    positive: Option<Vec<PromptVal>>,
    negative: Option<Vec<PromptVal>>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(untagged)]
pub enum PromptVal {
    MysteriousNumber(u32),
    Val(String),
    None,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(untagged)]
// what is wrong with python developers
pub enum CuresedTextVal {
    Single(PromptVal),
    Vec(Vec<PromptVal>),
}

/// Exif struct for image formats other than png
#[derive(Deserialize, Debug, Serialize)]
pub struct ComfyExifJson {
    pub prompt: ComfyPrompt,
}
