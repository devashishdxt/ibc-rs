use regex::Regex;
use thiserror::Error;

const VALID_ID_PATTERN: &str = r"^[a-zA-Z0-9\.\_\+\-\#\[\]\<\>]+$";

#[derive(Debug, Error)]
pub enum IdentifierValidationError {
    #[error("identifier cannot be blank")]
    BlankIdentifier,
    #[error("identifier {0} cannot contain separator '/'")]
    CannotContainSeparator(String),
    #[error("identifier {0} must contain only alphanumeric or the following characters: '.', '_', '+', '-', '#', '[', ']', '<', '>'")]
    InvalidChars(String),
    #[error("identifier {id} has invalid length: {len}, must be between {min}-{max} characters")]
    InvalidLength {
        id: String,
        len: usize,
        min: usize,
        max: usize,
    },
}

fn is_valid_id(id: &str) -> bool {
    Regex::new(VALID_ID_PATTERN).unwrap().is_match(id)
}

fn validate_identifier(id: &str, min: usize, max: usize) -> Result<(), IdentifierValidationError> {
    if id.trim().is_empty() {
        return Err(IdentifierValidationError::BlankIdentifier);
    }

    if id.contains("/") {
        return Err(IdentifierValidationError::CannotContainSeparator(
            id.to_owned(),
        ));
    }

    let id_len = id.len();

    if id_len < min || id_len > max {
        return Err(IdentifierValidationError::InvalidLength {
            id: id.to_owned(),
            len: id_len,
            min,
            max,
        });
    }

    if !is_valid_id(id) {
        return Err(IdentifierValidationError::InvalidChars(id.to_owned()));
    }

    Ok(())
}

/// Validator function for Client identifiers. A valid Identifier must be between 9-64 characters and only contain
/// alphanumeric and some allowed special characters
pub fn validate_client_id(id: &str) -> Result<(), IdentifierValidationError> {
    validate_identifier(id, 9, 64)
}
