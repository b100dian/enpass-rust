use rusqlite::{Connection, Result};
use std::fmt;

pub struct VaultCommand {
    connection: Connection,
}

pub struct ListItem {
    id: i32,
    title: String,
}

impl fmt::Display for ListItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.id, self.title)
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
}
