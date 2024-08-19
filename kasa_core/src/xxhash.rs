use std::{fs::File, path::Path};

#[inline]
pub fn streaming_xxhash(path: &Path) -> u128 {
    let mut file = File::open(path).unwrap();
    let mut buf = [0u8; 4096];
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
    loop {
        match std::io::Read::read(&mut file, &mut buf) {
            Ok(0) => {
                // end of file
                break;
            }
            Ok(len) => {
                hasher.update(&buf[..len]);
            }
            Err(error) => match error.kind() {
                std::io::ErrorKind::UnexpectedEof => {
                    println!("unexpected end of file reached");
                    break;
                }
                _ => panic!("error reading chunk: {}", error),
            },
        }
    }

    let hash = hasher.digest128();
    hash
}
