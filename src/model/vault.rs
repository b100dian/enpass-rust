use hex::encode;
use pbkdf2;
use rusqlite::{Connection, Result};
use sha2;
use std::fs::File;
use std::io::Read;

type Key64 = [u8; 64];

pub struct Vault {
    path: std::path::PathBuf,
    salt: [u8; 16],
}

impl Vault {
    pub fn new(path: std::path::PathBuf) -> std::io::Result<Vault> {
        let mut db_file = File::open(&path)?;
        let mut salt = [0; 16];
        db_file.read_exact(&mut salt)?;
        Ok(Vault { path, salt })
    }

    fn derive_key(&self, password: &[u8]) -> Key64 {
        return pbkdf2::pbkdf2_hmac_array::<sha2::Sha512, 64>(password, &self.salt, 320000);
    }

    pub fn login(&self, password: &[u8]) -> Result<Connection> {
        let derived_key = self.derive_key(password);
        let hex_key = encode(&derived_key[0..32]);
        println!("Using key {}", &hex_key);
        let connection =
            Connection::open_with_flags(&self.path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let pragma_key = format!(r#"PRAGMA key = "x'{key}'";"#, key = hex_key);
        connection.execute_batch(&pragma_key)?;
        connection.pragma_update(None, "cipher_compatibility", "3")?;
        connection.pragma_update(None, "cipher_page_size", "1024")?;

        Ok(connection)
    }
}
