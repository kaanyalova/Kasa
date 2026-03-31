from gallery_dl.job import DownloadJob
import json
import hashlib

class KasaOutput():
    def __init__(self, job, on_progress=None):
         self.job = job
         self.on_progress = on_progress

    def start(self, path):
        pass
    def skip(self, path):
        pass
    def success(self, path):    
        pass
    def progress(self, bytes_total, bytes_downloaded, bytes_per_second):
        state = {
            "bytes_total": bytes_total or 0,
            "bytes_downloaded": bytes_downloaded or 0,
            "bytes_per_second": bytes_per_second or 0,
            "url_hash": self.job.url_hash,
            "url": self.job.url,
            "extractor": self.job.extractor.category
        }

        state_json = json.dumps(state)
        self.on_progress(state_json)

class KasaDownloadJob(DownloadJob):
    def __init__(self, url, parent=None, on_progress = None):
        super().__init__(url, parent)
        self.output_paths = []
        # this is already set by the super but whatever 
        self.out = KasaOutput(self, on_progress)
        self.is_done = False
        self.url_hash = hashlib.sha1(url.encode("utf-8")).hexdigest()
        self.url = url
       

    
    def handle_url(self, url, kwdict):
        super().handle_url(url, kwdict)
        self.output_paths.append(self.pathfmt.path)


    def handle_finalize(self):
        super().handle_finalize()
        self.is_done = True

