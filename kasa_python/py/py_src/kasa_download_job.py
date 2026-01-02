from gallery_dl.job import DownloadJob
import json


class KasaOutput():
    def start(self, path):
        pass
    def skip(self, path):
        pass
    def success(self, path):    
        pass
    def progress(self, bytes_total, bytes_downloaded, bytes_per_second) -> str:
        return json.dumps({
            bytes_total: bytes_total,
            bytes_downloaded: bytes_downloaded,
            bytes_per_second: bytes_per_second
        })

class KasaDownloadJob(DownloadJob):
    def __init__(self, url, parent=None):
        super().__init__(url, parent)
        self.output_paths = []
        # this is already set by the super but whatever 
        self.output = KasaOutput()
    
    def handle_url(self, url, kwdict):
        super().handle_url(url, kwdict)
        self.output_paths.append(self.pathfmt.path)

