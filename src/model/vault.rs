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

pub struct VaultConnection {
    connection: Connection,
}

impl Vault {
    pub fn new(path: std::path::PathBuf) -> std::io::Result<Vault> {
        let mut db_file = File::open(&path)?;
        let mut salt = [0; 16];
        db_file.read_exact(&mut salt)?;
        Ok(Vault { path, salt })
    }

    pub fn salt(&self) -> [u8; 16] {
        self.salt
    }

    fn derive_key(&self, password: &[u8]) -> Key64 {
        return pbkdf2::pbkdf2_hmac_array::<sha2::Sha512, 64>(password, &self.salt, 320000);
    }

    pub fn login(&self, password: &[u8]) -> Result<Connection> {
        let derived_key = self.derive_key(password);

        let hex_key = encode(derived_key);
        let result =
            Connection::open_with_flags(&self.path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        result.execute_batch("PRAGMA CIPHER_COMPATIBILITY=3")?;
        result.execute_batch(format!(r#"PRAGMA KEY="x'{key}'""#, key = hex_key).as_str())?;
        Ok(result)
    }
}

impl VaultConnection {
    pub fn new(connection: Connection) -> VaultConnection {
        return VaultConnection { connection };
    }
}
