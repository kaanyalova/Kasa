use std::{io::Cursor, path::Path, vec};

use diffy::{Patch, apply};
use flate2::bufread::GzDecoder;
use tar::Archive;

fn main() {
    struct Source {
        url: String,
        path: String,
    }

    impl Source {
        fn new(url: &str, path: &str) -> Self {
            Source {
                url: url.to_string(),
                path: path.to_string(),
            }
        }

        fn download_and_extract(&self) {
            if self.does_source_exist() {
                return;
            }

            let archive = reqwest::blocking::get(&self.url).unwrap().bytes().unwrap();
            let cursor = Cursor::new(archive);
            let tar = GzDecoder::new(cursor);
            let mut archive = Archive::new(tar);
            archive
                .unpack(format!("py/dependencies/{}", self.path))
                .unwrap();
        }

        fn does_source_exist(&self) -> bool {
            Path::new(&format!("py/dependencies/{}", &self.path)).is_dir()
        }
    }

    struct _Patch {
        file_path: String,
        patch_path: String,
    }

    impl _Patch {
        fn new(file_path: &str, patch_path: &str) -> Self {
            _Patch {
                file_path: file_path.to_string(),
                patch_path: patch_path.to_string(),
            }
        }

        fn patch(&self) {
            let original = std::fs::read_to_string(&self.file_path).unwrap();
            let patch_file = std::fs::read_to_string(&self.patch_path).unwrap();
            let patch = Patch::from_str(&patch_file).unwrap();
            let apply = apply(&original, &patch);

            // dumb way of skipping already applied patches, will cause problems if patching actually fails
            match apply {
                Ok(patch) => {
                    std::fs::write(&self.file_path, patch).unwrap();
                }
                Err(e) => {
                    dbg!(e.to_string());
                }
            }

            //if let Ok(patched) = {
            //
            //} else if  {
        }
    }

    let sources = vec![
        Source::new(
            "https://github.com/mikf/gallery-dl/releases/download/v1.29.3/gallery_dl-1.29.3.tar.gz",
            "gallery-dl",
        ),
        Source::new(
            "https://github.com/jawah/charset_normalizer/releases/download/3.4.0/charset_normalizer-3.4.0.tar.gz",
            "charset_normalizer",
        ),
        Source::new(
            "https://github.com/psf/requests/releases/download/v2.32.3/requests-2.32.3.tar.gz",
            "requests",
        ),
        Source::new(
            "https://github.com/urllib3/urllib3/releases/download/2.2.3/urllib3-2.2.3.tar.gz",
            "urllib3",
        ),
        Source::new(
            "https://github.com/kjd/idna/releases/download/v3.10/idna-3.10.tar.gz",
            "idna",
        ),
    ];

    for source in sources {
        source.download_and_extract();
    }

    let patches = vec![
        _Patch::new(
            "py/dependencies/charset_normalizer/charset_normalizer-3.4.0/charset_normalizer/utils.py",
            "py/patches/fix_broken_multibytecodec_import.diff",
        ),
        _Patch::new(
            "py/dependencies/gallery-dl/gallery_dl-1.29.3/gallery_dl/job.py",
            "py/patches/add_output_paths.diff",
        ),
    ];

    for patch in patches {
        patch.patch();
    }
}
