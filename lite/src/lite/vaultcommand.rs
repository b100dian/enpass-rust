use std::fmt;

use super::vaulterror::{VaultError, VaultItemNotFound};
use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit,
};
use rusqlite::{Connection, Result, Row};
use totp_rfc6238::TotpGenerator;

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

pub struct KeyValue {
    pub key: String,
    pub value: String,
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

    fn decrypt_password(
        password: String,
        uuid: &String,
        item_key: &Vec<u8>,
    ) -> Result<String, VaultError> {
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
                    password: VaultCommand::decrypt_password(
                        matched.password,
                        &item.uuid,
                        &item.key,
                    )?,
                })
            } else {
                let vault_error = VaultItemNotFound { id: *id };
                Err(VaultError::VaultItemNotFound(vault_error))
            }
            // TODO v These ^ look so much the same.
        } else {
            let vault_error = VaultItemNotFound { id: *id };
            Err(VaultError::VaultItemNotFound(vault_error))
        }
    }

    pub fn dump(self, id: &u32) -> Result<Vec<KeyValue>, VaultError> {
        let item = self.item_details(id)?;
        let mut stmt = self.connection.prepare(
            "select type, value from itemfield where item_uuid = (?1) order by \"order\" asc",
        )?;
        let results = stmt.query_map([&item.uuid], |row: &Row| -> Result<KeyValue> {
            Ok(KeyValue {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        });

        let dump = results?
            .map(|row| -> Result<KeyValue, VaultError> {
                let field = row.unwrap();
                Ok(KeyValue {
                    key: field.key.clone(),
                    value: match field.key.as_str() {
                        "password" => match field.value.as_str() {
                            "" => "".to_string(),
                            _ => {
                                VaultCommand::decrypt_password(field.value, &item.uuid, &item.key)?
                            }
                        },
                        "totp" => {
                            let totp_key = base32::decode(
                                base32::Alphabet::Rfc4648Lower { padding: false },
                                &field.value,
                            );
                            match totp_key {
                                Some(key) => {
                                    let totp = TotpGenerator::new().build();
                                    format!("{} => {}", field.value, totp.get_code(&key))
                                }
                                None => field.value,
                            }
                        }
                        _ => field.value,
                    },
                })
            })
            .filter(|row| match row {
                Ok(field) => !field.value.is_empty(),
                _ => false,
            })
            .map(|x| x.unwrap());
        Ok(dump.collect())
    }
}
