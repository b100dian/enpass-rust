use std::fs::File;
use std::io::Read;

pub struct Vault {
    path: std::path::PathBuf,
    salt: [u8; 16],
}

impl Vault {
    pub fn new(path: std::path::PathBuf) -> std::io::Result<Vault> {
        // open file
        match File::open(&path) {
            Err(e) => Err(e),
            Ok(mut dbfile) => {
                let mut salt = [0; 16];
                match dbfile.read_exact(&mut salt) {
                    Err(e) => Err(e),
                    Ok(()) => Ok(Vault { path, salt }),
                }
            }
        }
    }

    pub fn salt(&self) -> [u8; 16] {
        self.salt
    }
}
