//! Native CUID2 construction with SHA3-512, clock, counter, fingerprint and ring entropy.
//!
//! Algorithm derived from @paralleldrive/cuid2 3.3.0 (MIT, Eric Elliott and contributors):
//! <https://github.com/paralleldrive/cuid2/blob/main/src/index.js>. Native entropy replaces
//! JS random callbacks; a random host fingerprint replaces JavaScript global-object keys.

use crate::{SchemaError, SchemaErrorKind, SchemaResult};
use ring::rand::{SecureRandom, SystemRandom};
use sha3::{Digest, Sha3_512};
use std::{
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

/// One thread-safe CUID2 host instance. Each ID starts with a letter and defaults to 24 chars.
///
/// # Examples
/// ```
/// use stackpress_framework_schema::providers::CuidGenerator;
/// let ids = CuidGenerator::new()?;
/// assert_eq!(ids.generate()?.len(), 24);
/// # Ok::<(), stackpress_framework_schema::SchemaError>(())
/// ```
pub struct CuidGenerator {
    /// Randomized host-local identity mixed into every hash, never exposed directly.
    fingerprint: String,
    /// Monotonic process-local sequence; relaxed ordering suffices for distinct reservations.
    counter: AtomicU64,
    /// Checked CUID2 output length from 2 through 32.
    length: usize,
}
impl CuidGenerator {
    /// Construct a default 24-character generator using ring OS randomness.
    /// # Errors
    /// Returns an entropy failure instead of falling back to a deterministic generator.
    pub fn new() -> SchemaResult<Self> {
        Self::with_length(24)
    }
    /// Construct a CUID2 length from 2 through 32, preserving the source maximum.
    /// # Errors
    /// Refuses an invalid length or unavailable entropy.
    pub fn with_length(length: usize) -> SchemaResult<Self> {
        // Typed Rust construction checks the documented CUID2 length range before randomness.
        if !(2..=32).contains(&length) {
            return Err(error("invalid CUID2 length"));
        }
        Self::with_source_length(length)
    }
    /// Original init accepts 0/1 despite documenting a two-character minimum.
    /// Keep that source-default seam separate from the existing checked native constructor.
    pub(super) fn with_source_length(length: usize) -> SchemaResult<Self> {
        if length > 32 {
            return Err(SchemaError::from_source_message(
                SchemaErrorKind::Codec,
                format!("Length must be between 2 and 32. Received: {length}"),
            ));
        }
        let fingerprint = hash36(&entropy(32)?).chars().take(32).collect();
        let counter = (random_fraction()? * 476_782_367.0).floor() as u64;
        Ok(Self {
            fingerprint,
            counter: AtomicU64::new(counter),
            length,
        })
    }
    /// Generate one owned ID without a mutex or borrowed runtime state.
    /// # Errors
    /// Propagates clock/entropy failures and exhausted native sequence values.
    pub fn generate(&self) -> SchemaResult<String> {
        // Reserve the sequence once, including failed attempts, so threads cannot reuse it.
        let counter = self
            .counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| error("CUID2 counter exhausted"))?;
        let first = (b'a' + (random_fraction()? * 26.0).floor() as u8) as char;
        let milliseconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| error("CUID2 clock precedes epoch").with_cause(e))?
            .as_millis();
        let salt = entropy(self.length)?;
        Ok(format_id(
            milliseconds,
            &salt,
            counter,
            &self.fingerprint,
            first,
            self.length,
        ))
    }
}
/// Generate a default CUID2 using a lazily shared host fingerprint and atomic counter.
/// # Errors
/// Propagates native randomness/clock failures; failed initialization may be retried.
pub fn cuid() -> SchemaResult<String> {
    static GENERATOR: OnceLock<CuidGenerator> = OnceLock::new();
    // Fallible initialization occurs before publishing; a racing loser uses the winning host.
    if GENERATOR.get().is_none() {
        let _ = GENERATOR.set(CuidGenerator::new()?);
    }
    GENERATOR
        .get()
        .ok_or_else(|| error("CUID2 initialization failed"))?
        .generate()
}
/// Match the reference's first-letter and double-dropped base36 hash construction.
pub(super) fn format_id(
    time: u128,
    salt: &str,
    counter: u64,
    fingerprint: &str,
    first: char,
    length: usize,
) -> String {
    let input = format!(
        "{}{}{}{}",
        radix36(time),
        salt,
        radix36(counter.into()),
        fingerprint
    );
    // hash36 already drops one histogram-biased digit; init drops another via substring(1).
    format!(
        "{first}{}",
        hash36(&input)
            .chars()
            .skip(usize::from(length != 0))
            .take(if length == 0 { 1 } else { length - 1 })
            .collect::<String>()
    )
}
/// Hash UTF-8 input and convert the full big-endian SHA3-512 integer to radix 36.
fn hash36(input: &str) -> String {
    let mut number = Sha3_512::digest(input.as_bytes()).to_vec();
    let mut digits = Vec::new();
    // Long division avoids a new bigint dependency and preserves all 512 integer bits.
    while number.iter().any(|&b| b != 0) {
        let mut remainder = 0_u16;
        for byte in &mut number {
            let value = remainder * 256 + u16::from(*byte);
            *byte = (value / 36) as u8;
            remainder = value % 36;
        }
        digits.push(b"0123456789abcdefghijklmnopqrstuvwxyz"[usize::from(remainder)] as char);
    }
    digits.reverse();
    digits.into_iter().skip(1).collect()
}
/// Convert clock/counter integers exactly, without floating point formatting.
fn radix36(mut number: u128) -> String {
    let mut out = Vec::new();
    loop {
        out.push(b"0123456789abcdefghijklmnopqrstuvwxyz"[(number % 36) as usize] as char);
        number /= 36;
        if number == 0 {
            break;
        }
    }
    out.into_iter().rev().collect()
}
/// Generate the same radix-36 salt alphabet as floor(random()*36).
fn entropy(length: usize) -> SchemaResult<String> {
    (0..length)
        .map(|_| {
            random_fraction().map(|n| {
                b"0123456789abcdefghijklmnopqrstuvwxyz"[(n * 36.0).floor() as usize] as char
            })
        })
        .collect()
}
/// Native counterpart of CUID2 3.3.0's random u32 divided by 2^32.
fn random_fraction() -> SchemaResult<f64> {
    let mut bytes = [0; 4];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| error("CUID2 randomness unavailable"))?;
    Ok(f64::from(u32::from_ne_bytes(bytes)) / 4_294_967_296.0)
}
/// Keep rejected input and native entropy internals out of diagnostics.
fn error(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Codec, context)
}

#[cfg(test)]
mod source_lengths {
    use super::*;
    #[test]
    fn original_parameterized_zero_one_and_maximum_vectors() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("cuid-lengths.json")).unwrap();
        for row in fixture["vectors"].as_array().unwrap() {
            let length = row["length"].as_u64().unwrap() as usize;
            if let Some(expected) = row["value"].as_str() {
                assert_eq!(
                    format_id(
                        1700000000123,
                        &"i".repeat(length),
                        1,
                        "source-test",
                        'n',
                        length
                    ),
                    expected
                );
                assert_eq!(
                    CuidGenerator::with_source_length(length)
                        .unwrap()
                        .generate()
                        .unwrap()
                        .len(),
                    expected.len()
                );
            } else {
                assert_eq!(
                    CuidGenerator::with_source_length(length)
                        .err()
                        .unwrap()
                        .to_string(),
                    row["error"].as_str().unwrap()
                );
            }
        }
        assert!(CuidGenerator::with_length(0).is_err());
        assert!(CuidGenerator::with_length(1).is_err());
    }
}
