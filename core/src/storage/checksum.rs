use crate::common::error::{EngineError, Result};
use md5::Md5;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub enum HashAlgorithm {
    Sha256,
    Md5,
}

pub struct ChecksumValidator;

impl ChecksumValidator {
    /// Compute hex hash string of a target file.
    pub fn verify_file(path: &Path, expected_hex: &str, algorithm: HashAlgorithm) -> Result<bool> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 64 * 1024];

        let computed_hex = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Md5 => {
                let mut hasher = Md5::new();
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                format!("{:x}", hasher.finalize())
            }
        };

        let matches = computed_hex.eq_ignore_ascii_case(expected_hex);
        if !matches {
            return Err(EngineError::ChecksumMismatch {
                expected: expected_hex.to_string(),
                actual: computed_hex,
            });
        }
        Ok(true)
    }
}
