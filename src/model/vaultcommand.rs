use rusqlite::{Connection, Result};
use std::fmt;

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

    fn item_uuid(&self, id: &u32) -> Result<String, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT uuid FROM item WHERE id = (?1)")?;
        let mut rows = stmt.query([&id])?;
        match rows.next()? {
            Some(row) => row.get(0),
            None => Err(rusqlite::Error::InvalidQuery),
        }
    }

    pub fn password(&self, id: &u32) -> Result<Password, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("
                select item1.value as username, item2.value as password from itemfield as item1 inner join itemfield as item2
                on item1.item_uuid = item2.item_uuid where item1.item_uuid = (?1) and item1.type='username' and item2.type = 'password'")?;
        let results = stmt.query_map([self.item_uuid(&id)?], |row| -> Result<Password> {
            Ok(Password {
                username: row.get(0)?,
                password: row.get(1)?,
            })
        })?;
        match results.last() {
            Some(first) => first,
            None => Err(rusqlite::Error::InvalidQuery),
        }
    }
}
