use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit,
};
use hex::FromHexError;
use rusqlite::{Connection, Result, Row};
use sha2::digest::InvalidLength;
use std::{error::Error, fmt, string::FromUtf8Error};

pub struct VaultCommand {
    connection: Connection,
}

pub struct ListItem {
    id: i32,
    title: String,
}

pub struct Password {
    username: String,
    password: String,
}

#[derive(Debug)]
pub struct VaultItemNotFound {
    id: u32,
}

impl fmt::Display for VaultItemNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item {} not found", self.id)
    }
}

impl Error for VaultItemNotFound {}

#[derive(thiserror::Error)]
pub enum VaultError {
    #[error("Failed to convert utf8")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Sqlite failure")]
    SqliteError(#[from] rusqlite::Error),
    #[error("Invalid length")]
    InvalidLength(#[from] InvalidLength),
    #[error("Error getting hex data")]
    FromHexError(#[from] FromHexError),
    #[error("AES GCM error")]
    AesGcmError(#[from] aes_gcm::Error),
    #[error("Internal vault error")]
    VaultItemNotFound(#[from] VaultItemNotFound),
}

impl std::fmt::Debug for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{self}")?;
        if let Some(e) = self.source() {
            writeln!(f, "\tCaused by: {e:?}")?;
        }
        Ok(())
    }
}

impl fmt::Display for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.id, self.title)
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.username, self.password)
    }
}

struct ItemDetails {
    uuid: String,
    key: Vec<u8>,
}

impl VaultCommand {
    pub fn new(connection: Connection) -> VaultCommand {
        return VaultCommand { connection };
    }

    pub fn list(&self) -> Result<Vec<ListItem>, rusqlite::Error> {
        let mut stmt = self.connection.prepare("SELECT id, title FROM item")?;

        let results = stmt.query_map([], |row| -> Result<ListItem> {
            Ok(ListItem {
                id: row.get(0)?,
                title: row.get(1)?,
            })
        })?;
        results.collect()
    }

    fn item_details(&self, id: &u32) -> Result<ItemDetails, VaultError> {
        let mut stmt = self
            .connection
            .prepare("SELECT uuid, key FROM item WHERE id = (?1)")?;
        let mut rows = stmt.query([&id])?;
        match rows.next()? {
            Some(row) => Ok(ItemDetails {
                uuid: row.get(0)?,
                key: row.get::<_, Vec<u8>>(1)?,
            }),
            None => {
                let vault_item_not_found = VaultItemNotFound { id: *id };
                Err(VaultError::VaultItemNotFound(vault_item_not_found))
            }
        }
    }

    fn decrypt(password: String, uuid: String, item_key: Vec<u8>) -> Result<String, VaultError> {
        let key = &item_key[..32];
        let nonce = &item_key[32..];
        let cipher = Aes256Gcm::new_from_slice(key)?;
        let header = hex::decode(uuid.replace("-", ""))?;
        let value = hex::decode(password)?;
        let result = cipher.decrypt(
            nonce.into(),
            Payload {
                msg: &value[..],
                aad: &header[..],
            },
        )?;
        Ok(String::from_utf8(result)?)
    }

    pub fn password(&self, id: &u32) -> Result<Password, VaultError> {
        let item = self.item_details(id)?;
        let mut stmt = self
            .connection
            .prepare("
                select item1.value as username, item2.value as password from itemfield as item1 inner join itemfield as item2
                on item1.item_uuid = item2.item_uuid where item1.item_uuid = (?1) and item1.type='username' and item2.type = 'password'")?;
        let results = stmt.query_map([&item.uuid], |row: &Row| -> Result<Password> {
            Ok(Password {
                username: row.get(0)?,
                password: row.get(1)?,
            })
        })?;
        if let Some(result) = results.last() {
            if let Ok(matched) = result {
                Ok(Password {
                    username: matched.username,
                    password: VaultCommand::decrypt(matched.password, item.uuid, item.key)?,
                })
            } else {
                let vault_error = VaultItemNotFound { id: *id };
                Err(VaultError::VaultItemNotFound(vault_error))
            }
        } else {
            let vault_error = VaultItemNotFound { id: *id };
            Err(VaultError::VaultItemNotFound(vault_error))
        }
    }
}
