//! Byte-compatible schema helpers: deterministic SHAKE256 and seed-derived AES-256-CBC.

use crate::{SchemaError, SchemaErrorKind, SchemaResult};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use base64::{Engine, engine::general_purpose::STANDARD};
use ring::digest;
use sha3::{
    Shake256,
    digest::{ExtendableOutput, Update, XofReader},
};

/// Source `hash(string)`: SHAKE256 with 32 output bytes, lowercase hexadecimal.
/// This preserves existing credential/challenge bytes; it is not an Argon2 migration.
pub fn hash(value: &str) -> String {
    let mut state = Shake256::default();
    state.update(value.as_bytes());
    let mut bytes = [0; 32];
    state.finalize_xof().read(&mut bytes);
    hex(&bytes)
}
/// Source `encrypt(value, seed)`: UTF-8, deterministic AES-256-CBC/PKCS7, lowercase hex.
/// The key is the first 32 ASCII chars of SHA256(seed) **base64**, and IV its first 16.
pub fn encrypt(value: &str, seed: &str) -> String {
    let key = key(seed);
    let cipher = cbc::Encryptor::<aes::Aes256>::new((&key).into(), (&key[..16]).into());
    hex(&cipher.encrypt_padded_vec_mut::<Pkcs7>(value.as_bytes()))
}
/// Source `decrypt(encrypted, seed)`, including Node's permissive hex decoding prefix.
/// # Errors
/// Returns a codec error for invalid block length or PKCS7 padding. Invalid UTF-8 is
/// decoded with replacement characters as Buffer.toString() does, not rejected.
pub fn decrypt(encrypted: &str, seed: &str) -> SchemaResult<String> {
    let key = key(seed);
    let bytes = unhex_prefix(encrypted);
    let cipher = cbc::Decryptor::<aes::Aes256>::new((&key).into(), (&key[..16]).into());
    let plain = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&bytes)
        .map_err(|_| SchemaError::new(SchemaErrorKind::Codec, "source AES-CBC decode failed"))?;
    Ok(String::from_utf8_lossy(&plain).into_owned())
}
/// Derive exactly the source's base64-text key; digest bytes/hex would change stored values.
fn key(seed: &str) -> [u8; 32] {
    let encoded = STANDARD.encode(digest::digest(&digest::SHA256, seed.as_bytes()).as_ref());
    let mut key = [0; 32];
    key.copy_from_slice(&encoded.as_bytes()[..32]);
    key
}
/// Encode lowercase hex without another external dependency.
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    bytes
        .iter()
        .flat_map(|b| {
            [
                DIGITS[usize::from(b >> 4)] as char,
                DIGITS[usize::from(b & 15)] as char,
            ]
        })
        .collect()
}
/// Node Buffer.from(hex) consumes complete valid pairs until the first invalid pair.
fn unhex_prefix(input: &str) -> Vec<u8> {
    input
        .as_bytes()
        .chunks_exact(2)
        .map_while(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some((high * 16 + low) as u8)
        })
        .collect()
}
