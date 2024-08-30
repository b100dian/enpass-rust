use hex::encode;
use memmap::{Mmap, MmapOptions};
use pbkdf2;
use rusqlite::{Connection, LoadExtensionGuard, Result};
use sha2;
use std::fs::File;
use std::os::unix::fs::FileExt;

use crate::lite::vaultmetadata::{VaultInfo, VaultOpenMode};

use super::vaulterror::VaultError;

type Key64 = [u8; 64];

const MMAPPED_DB_OFFSET: u32 = 0x400;

pub struct Vault {
    salt: [u8; 16],
    mmap: Option<Mmap>,
    vault_info: VaultInfo,
}

impl Vault {
    pub fn new(path: std::path::PathBuf) -> std::io::Result<Vault> {
        let vault_info = VaultInfo::new(path)?;
        // TODO assert that algos are supported
        let db_file = File::open(&vault_info.path)?;
        let mut salt = [0; 16];
        let offset = match vault_info.open_method {
            VaultOpenMode::Direct => 0,
            VaultOpenMode::Mmapped => MMAPPED_DB_OFFSET,
        };
        db_file.read_at(&mut salt, offset as u64)?;
        let hex_salt = encode(&salt[0..16]);

        log::debug!("Hexsalt:{}", hex_salt);
        Ok(Vault {
            salt,
            mmap: None,
            vault_info,
        })
    }

    fn derive_key(&self, password: &[u8]) -> Key64 {
        return pbkdf2::pbkdf2_hmac_array::<sha2::Sha512, 64>(
            password,
            &self.salt,
            self.vault_info.metadata.kdf_iter,
        );
    }

    fn load_memvfs(&self, conn: &Connection) -> Result<()> {
        unsafe {
            let _guard = LoadExtensionGuard::new(conn)?;
            conn.load_extension("libmemvfs", None)
        }
    }

    pub fn login(&mut self, password: &[u8]) -> Result<Connection, VaultError> {
        let derived_key = self.derive_key(password);
        let hex_key = encode(&derived_key[0..32]);
        log::debug!("Using key {}", &hex_key);

        let mut connection = Connection::open_with_flags(
            &self.vault_info.path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;

        match self.vault_info.open_method {
            VaultOpenMode::Mmapped => match self.load_memvfs(&connection) {
                Ok(_) => {
                    drop(connection);
                    let file = File::open(&self.vault_info.path).unwrap(); // TODO?
                    self.mmap = Some(unsafe {
                        MmapOptions::new().map(&file).unwrap() /*TODO?*/
                    });
                    let mmap = self.mmap.as_ref().unwrap();
                    let path = format!(
                        "file:mmaped?ptr={x:p}&sz={len}",
                        x = unsafe { mmap.as_ptr().byte_offset(MMAPPED_DB_OFFSET as isize) },
                        len = mmap.len() - MMAPPED_DB_OFFSET as usize
                    );
                    log::debug!("Using connection string {}", path);
                    connection = Connection::open_with_flags(
                        path,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                    )?;
                }
                Err(_) => {
                    log::error!("Need memvfs.c extension. See https://sqlite.org/loadext.html#compiling_a_loadable_extension");
                }
            },
            VaultOpenMode::Direct => { /* fallthrough */ }
        }

        let pragma_key = format!(r#"PRAGMA key = "x'{key}'";"#, key = hex_key);
        connection.execute_batch(&pragma_key)?;
        connection.pragma_update(None, "cipher_compatibility", "3")?;
        connection.pragma_update(None, "cipher_page_size", "1024")?;

        Ok(connection)
    }
}
