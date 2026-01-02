from gallery_dl.job import DownloadJob
import json
import hashlib

class KasaOutput():
    def __init__(self):
         self.state = {
            "bytes_total": 0,
            "bytes_downloaded": 0,
            "bytes_per_second": 0
        }
        

    def start(self, path):
        pass
    def skip(self, path):
        pass
    def success(self, path):    
        pass
    def progress(self, bytes_total, bytes_downloaded, bytes_per_second):
        self.state = {
            "bytes_total": bytes_total or 0,
            "bytes_downloaded": bytes_downloaded or 0,
            "bytes_per_second": bytes_per_second or 0
        }

class KasaDownloadJob(DownloadJob):
    def __init__(self, url, parent=None):
        super().__init__(url, parent)
        self.output_paths = []
        # this is already set by the super but whatever 
        self.out = KasaOutput()
        self.is_done = False
        self.url_hash = hashlib.sha1(url.encode("utf-8")).hexdigest() 
        
       

    
    def handle_url(self, url, kwdict):
        super().handle_url(url, kwdict)
        self.output_paths.append(self.pathfmt.path)


    def handle_finalize(self):
        super().handle_finalize()
        self.is_done = True

