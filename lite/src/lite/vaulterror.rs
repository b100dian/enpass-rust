use hex::FromHexError;
use sha2::digest::InvalidLength;
use std::{error::Error, fmt, string::FromUtf8Error};

#[derive(Debug)]
pub struct VaultItemNotFound {
    pub id: u32,
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
