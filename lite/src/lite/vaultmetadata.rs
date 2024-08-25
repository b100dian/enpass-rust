use core::str;
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub const SUPPORTED_KDF_ALGOS: &[&str; 1] = &["pbkdf2"];
pub const SUPPORTED_ENCRYPTION_ALGOS: &[&str; 1] = &["aes-256-cbc"];

pub enum VaultOpenMode {
    Direct,
    Mmapped,
}

pub struct VaultInfo {
    pub path: PathBuf,
    pub open_method: VaultOpenMode,
    pub metadata: VaultMetadata,
}

impl VaultInfo {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let default = |path| {
            log::warn!("No metadata detected, using defaults.");
            Ok(Self {
                path,
                open_method: VaultOpenMode::Direct,
                metadata: Default::default(),
            })
        };
        match path.extension() {
            Some(extension) => match extension.to_str() {
                Some("json") => {
                    let json = VaultInfo::fetch_string(&path)?;
                    let metadata = VaultInfo::parse_metadata(json);
                    Ok(Self {
                        path: path.with_extension("enpassdb"),
                        open_method: VaultOpenMode::Direct,
                        metadata,
                    })
                }
                Some("enpassdbsync") => {
                    let json = VaultInfo::fetch_string(&path)?;
                    let metadata = VaultInfo::parse_metadata(json);
                    Ok(Self {
                        path,
                        open_method: VaultOpenMode::Mmapped,
                        metadata,
                    })
                }
                Some("enpassdb") => {
                    let metadata = match VaultInfo::fetch_string(path.with_extension("json")) {
                        Ok(json) => VaultInfo::parse_metadata(json),
                        Err(_) => {
                            log::warn!(
                                "Missing json metadata for {}, using defaults.",
                                path.display()
                            );
                            VaultMetadata::default()
                        }
                    };
                    Ok(Self {
                        path,
                        open_method: VaultOpenMode::Direct,
                        metadata,
                    })
                }
                Some(x) => {
                    log::debug!("Unknown extension {}", x);
                    default(path)
                }
                None => default(path),
            },
            None => default(path),
        }
    }

    /// Reads the first 0x400 bytes and converts them to string till the first NULL.
    fn fetch_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let mut file = File::open(&path)?;
        let mut buffer = [0; 0x400];
        let read = file.read(&mut buffer)?;
        let null_marker = buffer.iter().position(|&c| c == b'\0').unwrap_or(read);
        Ok(unsafe { String::from_utf8_unchecked(buffer[0..null_marker].to_vec()) })
    }

    fn parse_metadata(json: String) -> VaultMetadata {
        log::debug!("JSON\n{}", json);
        let default = VaultMetadata::default();
        match json::parse(json.as_str()) {
            Ok(mut parsed) => VaultMetadata {
                encryption_algo: match parsed["encryption_algo"].take_string() {
                    Some(s) => s,
                    None => {
                        log::warn!("Missing encryption_algo, using default.");
                        default.encryption_algo
                    }
                },
                have_keyfile: match parsed["have_keyfile"].as_isize() {
                    Some(x) => x != 0,
                    None => {
                        log::warn!("Missing have_keyfile, using default.");
                        default.have_keyfile
                    }
                },
                kdf_algo: match parsed["kdf_algo"].take_string() {
                    Some(s) => s,
                    None => {
                        log::warn!("Missing kdf_algo, using default.");
                        default.kdf_algo
                    }
                },
                kdf_iter: match parsed["kdf_iter"].as_u32() {
                    Some(x) => x,
                    None => {
                        log::warn!("Missing kdf_iter, using default.");
                        default.kdf_iter
                    }
                },
                vault_uuid: match parsed["vault_uuid"].take_string() {
                    Some(s) => s,
                    None => {
                        log::warn!("Missing vault_uuid, using default.");
                        default.vault_uuid
                    }
                },
                version: match parsed["version"].as_usize() {
                    Some(x) => x,
                    None => {
                        log::warn!("Missing version, using default.");
                        default.version
                    }
                },
            },
            Err(e) => {
                log::error!("{}", e);
                log::warn!("Using default values for all vault metadata fields.");
                default
            }
        }
    }
}

// Fields from vault.json
pub struct VaultMetadata {
    pub encryption_algo: String,
    pub have_keyfile: bool,
    pub kdf_algo: String,
    pub kdf_iter: u32,
    pub vault_uuid: String,
    pub version: usize,
}

impl Default for VaultMetadata {
    fn default() -> Self {
        Self {
            encryption_algo: SUPPORTED_KDF_ALGOS[0].to_string(),
            have_keyfile: false,
            kdf_algo: SUPPORTED_KDF_ALGOS[0].to_string(),
            kdf_iter: 100000,
            vault_uuid: "primary".to_string(),
            version: 6,
        }
    }
}
