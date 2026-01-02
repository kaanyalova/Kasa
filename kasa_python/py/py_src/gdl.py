# from gallery_dl_ import InputManager, job, config
# from gallery_dl_.extractor.message import Message

from gallery_dl import InputManager, job, config
from gallery_dl.extractor.message import Message
import logging
import json
from kasa_download_job import KasaDownloadJob
jobs = []

def download(url: str, path: str, config_dir: str) -> str:
    config.load(files=[config_dir])
    config.set(("extractor",), "base-directory", path)
    # config.set(("extractor", "reddit"), "client-id", "")
    # config.set(
    #    ("extractor", "reddit"),
    #    "user-agent",
    #    "Python: <gallery_dl>:v1.0 (by /u/)",
    # )

    # options = json.loads(config_options_json)

    # for option in options:
    #    print(option)
    #    key_tuple = tuple(option["keys"])
    #    if len(key_tuple) < 2:
    #        # todo check from rust side instead
    #        raise AssertionError(
    #            "Gallery dl config options should have at least 2 keys"
    #        )

    # wtf is point of having an array for categories of keys but using another key for the options
    # config.set(key_tuple[:-1], key_tuple[-1], option["value"])
    # print(f"loaded config option, val= {option['value']}")

    _job = KasaDownloadJob(url)
    jobs.append(_job)

    # TODO check the status
    status = _job.run()

    # name

    if status != 0:
        raise Exception("Gallery-dl did not exit with 0, check the logs.")

    url_extractors = list(
        filter(lambda msg: msg[0] == Message.Url, list(_job.extractor))
    )

    directory_extractors = list(
        filter(lambda msg: msg[0] == Message.Directory, list(_job.extractor))
    )

    url_extractor_info = []

    if len(url_extractors) == 0:
        raise (
            AssertionError(
                "No URL extractor info was found, are you sure the post exists?"
            )
        )

    if len(_job.output_paths) != len(url_extractors):
        raise (
            AssertionError(
                f"The path count is not same as metadata count, length of job.output_paths was {len(_job.output_paths)} and lenght of url_extractors was {len(url_extractors)}"
            )
        )

    for i, extractor in enumerate(url_extractors):
        if extractor[0] == Message.Url:
            entry = {
                "extractor": _job.extractor.category,  # this is here so serde can recognize the tags
                "path": _job.output_paths[i],
                "url": extractor[1],
                "meta": extractor[2],
            }

            url_extractor_info.append(entry)

    info = {
        "extractor": _job.extractor.category,
        "base_url": url,
        "url_extractors": url_extractor_info,
        "dir_extractors": directory_extractors,
    }

    # datetime is not serializable without default=str
    return json.dumps(info, default=str)


def get_jobs_status() -> str:
    global jobs
    jobs = list(filter(lambda j: not j.is_done, jobs))

    jobs_output = {}

    for job in jobs:
        jobs_output[job.url_hash] = job.out.state

    return json.dumps(jobs_output)
        
