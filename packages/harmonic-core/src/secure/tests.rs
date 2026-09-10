use super::envelope::seal_with_material;
use super::*;
use crate::error::LogiscoreError;

const SALT: [u8; 16] = [0x21; 16];
const NONCE: [u8; 12] = [0x42; 12];

#[test]
fn secure_envelope_roundtrips_unicode_bytes() {
    let envelope =
        seal_with_material("秘密の楽譜 🎼".as_bytes(), "correct horse", SALT, NONCE).unwrap();
    assert_eq!(
        open(envelope.as_bytes(), "correct horse").unwrap(),
        "秘密の楽譜 🎼".as_bytes()
    );
}

#[test]
fn wrong_password_and_ciphertext_tampering_are_not_disclosed() {
    let envelope = seal_with_material(b"never reveal", "correct", SALT, NONCE).unwrap();
    assert!(matches!(
        open(envelope.as_bytes(), "wrong"),
        Err(LogiscoreError::AuthenticationFailed)
    ));
    let mut tampered = envelope.into_bytes();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert!(matches!(
        open(&tampered, "correct"),
        Err(LogiscoreError::AuthenticationFailed)
    ));
}

#[test]
fn envelope_validates_password_header_and_randomized_output() {
    assert!(matches!(
        seal(b"data", ""),
        Err(LogiscoreError::InvalidPassword)
    ));
    assert!(SecureEnvelope::from_bytes(vec![0; 51]).is_err());
    let first = seal(b"same", "password").unwrap();
    let second = seal(b"same", "password").unwrap();
    assert_ne!(first, second);
}
