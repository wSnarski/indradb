//! Utility functions.

use chrono::offset::Utc;
use chrono::DateTime;
use crate::errors::{ValidationError, ValidationResult};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use std::env;
use uuid::Uuid;

const TEMP_PATH_RANDOM_PART_LENGTH: usize = 8;


/// Gets the path to a file or directory within the temporary directory, in a
/// platform-independent manner. Note that this will panic if the path cannot
/// be formatted into UTF-8.
pub fn generate_temporary_path() -> String {
    let mut path = env::temp_dir();
    path.push(generate_random_secret(TEMP_PATH_RANDOM_PART_LENGTH));
    path.to_str()
        .expect("Expected to be able to parse the temp path into UTF-8")
        .to_string()
}

/// Generates a UUID v1. this utility method uses a shared context and node ID
/// to help ensure generated UUIDs are unique.
pub fn generate_uuid_v1() -> Uuid {
    Uuid::new_v4()
}

/// Generates a securely random string consisting of letters (uppercase and
/// lowercase) and digits.
pub fn generate_random_secret(count: usize) -> String {
    let mut chars = vec![];
    let options = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = OsRng::new().unwrap();

    for _ in 0..count {
        let c: u8 = *options.choose(&mut rng).unwrap();
        chars.push(c);
    }

    String::from_utf8(chars).unwrap()
}

/// Gets the next UUID that would occur after the given one.
///
/// # Arguments
///
/// * `uuid` - The input UUID.
///
/// # Errors
/// Returns a `ValidationError` if the input UUID is the great possible value
/// (i.e., FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF)
pub fn next_uuid(uuid: Uuid) -> ValidationResult<Uuid> {
    let mut bytes = *uuid.as_bytes();

    for i in (0..16).rev() {
        if bytes[i] < 255 {
            bytes[i] += 1;
            return Ok(Uuid::from_slice(&bytes[..]).unwrap());
        } else {
            bytes[i] = 0;
        }
    }

    Err(ValidationError::CannotIncrementUuid)
}

/// Gets the number of nanoseconds since unix epoch for a given datetime.
///
/// # Arguments
/// * `datetime` - The datetime to convert.
pub fn nanos_since_epoch(datetime: &DateTime<Utc>) -> u64 {
    let timestamp = datetime.timestamp() as u64;
    let nanoseconds = u64::from(datetime.timestamp_subsec_nanos());
    timestamp * 1_000_000_000 + nanoseconds
}

#[cfg(test)]
mod tests {
    use super::{generate_random_secret, generate_temporary_path, generate_uuid_v1, nanos_since_epoch, next_uuid};
    use chrono::{DateTime, NaiveDateTime, Utc};
    use core::str::FromStr;
    use regex::Regex;
    use uuid::Uuid;

    #[test]
    fn should_generate_temporary_path() {
        let first = generate_temporary_path();
        let second = generate_temporary_path();
        assert!(first.len() > 0);
        assert!(second.len() > 0);
        assert_ne!(first, second);
    }

    #[test]
    fn should_generate_new_uuid_v1() {
        let first = generate_uuid_v1();
        let second = generate_uuid_v1();
        assert_ne!(first, second);
    }

    #[test]
    fn should_generate_random_secret() {
        let secret = generate_random_secret(8);
        assert!(Regex::new(r"[a-zA-Z0-9]{8}").unwrap().is_match(&secret[..]));
    }

    #[test]
    fn should_generate_next_uuid() {
        let result = next_uuid(Uuid::from_str("16151dea-a538-4bf1-9559-851e256cf139").unwrap());
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Uuid::from_str("16151dea-a538-4bf1-9559-851e256cf13a").unwrap()
        );

        let from_uuid = Uuid::from_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();
        assert!(next_uuid(from_uuid).is_err());
    }

    #[test]
    fn should_generate_nanos_since_epoch() {
        let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 62), Utc);
        assert_eq!(nanos_since_epoch(&datetime), 61000000062);
    }
}
