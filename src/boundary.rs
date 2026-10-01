use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterError {
    pub field: &'static str,
    pub message: &'static str,
}

impl AdapterError {
    pub(crate) const fn new(field: &'static str, message: &'static str) -> Self {
        Self { field, message }
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for AdapterError {}

pub(crate) fn identifier(field: &'static str, value: &str) -> Result<(), AdapterError> {
    if value.is_empty()
        || value.len() > 96
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
    {
        return Err(AdapterError::new(
            field,
            "must be a lowercase ASCII identifier",
        ));
    }
    Ok(())
}

pub(crate) fn reference(field: &'static str, value: &str) -> Result<(), AdapterError> {
    if value.is_empty()
        || value.len() > 256
        || value.trim() != value
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(AdapterError::new(
            field,
            "must be a bounded opaque reference",
        ));
    }
    Ok(())
}

pub(crate) fn digest(field: &'static str, value: &str) -> Result<(), AdapterError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(AdapterError::new(
            field,
            "must be 64 lowercase hex characters",
        ));
    }
    Ok(())
}

pub(crate) fn github_name(field: &'static str, value: &str) -> Result<(), AdapterError> {
    if value.is_empty()
        || value.len() > 100
        || value.starts_with('.')
        || value.ends_with('.')
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(AdapterError::new(field, "contains an invalid GitHub name"));
    }
    Ok(())
}
