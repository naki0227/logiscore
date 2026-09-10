use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::ChaCha20Poly1305;
use zeroize::Zeroizing;

use crate::error::LogiscoreError;

const MAGIC: [u8; 4] = *b"LSSE";
const VERSION: u8 = 1;
const KDF_ID: u8 = 1;
const AEAD_ID: u8 = 1;
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 12;
const TAG_BYTES: usize = 16;
const HEADER_BYTES: usize = 4 + 1 + 1 + 1 + SALT_BYTES + NONCE_BYTES;
const MAX_PASSWORD_BYTES: usize = 1_024;
const MAX_PLAINTEXT_BYTES: usize = 36 * 1024 * 1024;
const ARGON2_MEMORY_KIB: u32 = 64 * 1024;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_LANES: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureEnvelope {
    bytes: Vec<u8>,
}

impl SecureEnvelope {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, LogiscoreError> {
        validate_envelope(&bytes)?;
        Ok(Self { bytes })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub fn seal(plaintext: &[u8], password: &str) -> Result<SecureEnvelope, LogiscoreError> {
    let mut salt = [0u8; SALT_BYTES];
    let mut nonce = [0u8; NONCE_BYTES];
    getrandom::fill(&mut salt).map_err(|_| LogiscoreError::SecureModeUnavailable)?;
    getrandom::fill(&mut nonce).map_err(|_| LogiscoreError::SecureModeUnavailable)?;
    seal_with_material(plaintext, password, salt, nonce)
}

pub fn open(envelope: &[u8], password: &str) -> Result<Vec<u8>, LogiscoreError> {
    validate_password(password)?;
    validate_envelope(envelope)?;
    let salt: &[u8; SALT_BYTES] = envelope[7..7 + SALT_BYTES]
        .try_into()
        .map_err(|_| invalid_envelope("salt is malformed"))?;
    let nonce_start = 7 + SALT_BYTES;
    let nonce: &[u8; NONCE_BYTES] = envelope[nonce_start..HEADER_BYTES]
        .try_into()
        .map_err(|_| invalid_envelope("nonce is malformed"))?;
    let key = derive_key(password, salt)?;
    let cipher = ChaCha20Poly1305::new((&*key).into());
    cipher
        .decrypt(
            nonce.into(),
            Payload {
                msg: &envelope[HEADER_BYTES..],
                aad: &envelope[..HEADER_BYTES],
            },
        )
        .map_err(|_| LogiscoreError::AuthenticationFailed)
}

pub(crate) fn seal_with_material(
    plaintext: &[u8],
    password: &str,
    salt: [u8; SALT_BYTES],
    nonce: [u8; NONCE_BYTES],
) -> Result<SecureEnvelope, LogiscoreError> {
    validate_password(password)?;
    if plaintext.len() > MAX_PLAINTEXT_BYTES {
        return Err(invalid_envelope("plaintext exceeds size limit"));
    }
    let mut header = Vec::with_capacity(HEADER_BYTES);
    header.extend_from_slice(&MAGIC);
    header.extend_from_slice(&[VERSION, KDF_ID, AEAD_ID]);
    header.extend_from_slice(&salt);
    header.extend_from_slice(&nonce);
    let key = derive_key(password, &salt)?;
    let cipher = ChaCha20Poly1305::new((&*key).into());
    let ciphertext = cipher
        .encrypt(
            (&nonce).into(),
            Payload {
                msg: plaintext,
                aad: &header,
            },
        )
        .map_err(|_| LogiscoreError::SecureModeUnavailable)?;
    header.extend_from_slice(&ciphertext);
    SecureEnvelope::from_bytes(header)
}

fn derive_key(
    password: &str,
    salt: &[u8; SALT_BYTES],
) -> Result<Zeroizing<[u8; 32]>, LogiscoreError> {
    let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_LANES, Some(32))
        .map_err(|_| LogiscoreError::SecureModeUnavailable)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut *key)
        .map_err(|_| LogiscoreError::SecureModeUnavailable)?;
    Ok(key)
}

fn validate_password(password: &str) -> Result<(), LogiscoreError> {
    if password.is_empty() || password.len() > MAX_PASSWORD_BYTES {
        return Err(LogiscoreError::InvalidPassword);
    }
    Ok(())
}

fn validate_envelope(bytes: &[u8]) -> Result<(), LogiscoreError> {
    if bytes.len() < HEADER_BYTES + TAG_BYTES {
        return Err(invalid_envelope("data is truncated"));
    }
    if bytes.len() > HEADER_BYTES + TAG_BYTES + MAX_PLAINTEXT_BYTES {
        return Err(invalid_envelope("data exceeds size limit"));
    }
    if bytes[..4] != MAGIC {
        return Err(invalid_envelope("magic mismatch"));
    }
    if bytes[4] != VERSION {
        return Err(LogiscoreError::UnsupportedVersion(format!(
            "secure envelope v{}",
            bytes[4]
        )));
    }
    if bytes[5] != KDF_ID || bytes[6] != AEAD_ID {
        return Err(invalid_envelope("unsupported cryptographic profile"));
    }
    Ok(())
}

fn invalid_envelope(message: &str) -> LogiscoreError {
    LogiscoreError::InvalidSecureEnvelope(message.to_owned())
}
