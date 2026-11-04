//! Source nanoid URL alphabet and byte masking, using the approved ring entropy source.
use crate::{SchemaError, SchemaErrorKind, SchemaResult};
use ring::rand::{SecureRandom, SystemRandom};

/// Nanoid's original URL-safe shuffled alphabet; 64 entries make masking unbiased.
pub const NANOID_ALPHABET: &str =
    "useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict";

/// Generate the source default 21-character nanoid.
/// # Errors
/// Propagates native randomness failure without falling back to deterministic bytes.
pub fn nanoid() -> SchemaResult<String> {
    nanoid_with_length(21)
}

/// Generate a source nanoid of the explicitly selected length, including zero.
/// # Errors
/// Propagates native randomness failure. Length must fit the native address space.
pub fn nanoid_with_length(length: usize) -> SchemaResult<String> {
    generate_with(length, |bytes| {
        SystemRandom::new()
            .fill(bytes)
            .map_err(|_| SchemaError::new(SchemaErrorKind::Codec, "nanoid randomness unavailable"))
    })
}

/// Keep entropy injection private to deterministic source-oracle and failure tests.
fn generate_with(
    length: usize,
    fill: impl FnOnce(&mut [u8]) -> SchemaResult<()>,
) -> SchemaResult<String> {
    // Native allocation failure is a typed provider error, corresponding to Node's RangeError.
    let allocation = || {
        SchemaError::new(
            SchemaErrorKind::Codec,
            "nanoid length exceeds native allocation",
        )
    };
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| allocation())?;
    bytes.resize(length, 0);
    fill(&mut bytes)?;
    // nanoid/index.js consumes its pool forwards, applying &63 to each byte.
    let mut result = String::new();
    result.try_reserve_exact(length).map_err(|_| allocation())?;
    result.extend(
        bytes
            .into_iter()
            .map(|byte| NANOID_ALPHABET.as_bytes()[usize::from(byte & 63)] as char),
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_node_byte_vectors_and_entropy_failure() {
        let vectors: serde_json::Value =
            serde_json::from_str(include_str!("nanoid-vectors.json")).unwrap();
        for row in vectors["vectors"].as_array().unwrap() {
            let bytes: Vec<u8> = row["bytes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_u64().unwrap() as u8)
                .collect();
            assert_eq!(
                generate_with(bytes.len(), |target| {
                    target.copy_from_slice(&bytes);
                    Ok(())
                })
                .unwrap(),
                row["id"].as_str().unwrap()
            );
        }
        assert!(
            generate_with(8, |_| Err(SchemaError::new(
                SchemaErrorKind::Codec,
                "owned entropy failure"
            )))
            .is_err()
        );
        assert_eq!(nanoid_with_length(0).unwrap(), "");
        assert_eq!(nanoid().unwrap().len(), 21);
        assert!(nanoid_with_length(usize::MAX).is_err());
    }
}
