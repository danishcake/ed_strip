///! Contains structures related to the type hints mechanism
use std::{ops::Deref, path::Path, str::FromStr};

use glob::Pattern;
use log::debug;
use serde::Deserialize;
use thiserror::Error;

// To deserialize Pattern we use the NewType pattern
pub struct GlobPattern(Pattern);

/// Implement Deserialize for GlobPattern so that patterns can be loaded
impl<'a> Deserialize<'a> for GlobPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let s = String::deserialize(deserializer)?;
        Pattern::from_str(&s)
            .map(GlobPattern)
            .map_err(serde::de::Error::custom)
    }
}

/// From allows a Pattern to be easily converted to a GlobPattern
/// This is primarily for unit tests
impl From<Pattern> for GlobPattern {
    fn from(value: Pattern) -> Self {
        GlobPattern(value)
    }
}

/// Deref allows the GlobPattern to be used like a Pattern
impl Deref for GlobPattern {
    type Target = Pattern;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An type hint via a pattern
#[derive(Deserialize)]
pub struct TypeHint {
    /// A glob pattern that matches file(s)
    pub pattern: GlobPattern,
    /// A language name to use
    pub language: String,
}

/// A type representing a type hints file
///
/// This corresponds to
///
/// {
///   pattern: string,
///   language: string
/// }[];
///
pub type TypeHints = Vec<TypeHint>;

/// Type hint load error
#[derive(Error, Debug)]
pub enum TypeHintLoadError {
    #[error("unable to load file: {0}")]
    StrippingError(#[from] std::io::Error),

    #[error("unable to parse file")]
    ParseError(#[from] serde_json::Error),
}

/// Loads the type hints file
///
/// # Arguments
/// * `path` - The path to the type hints file
pub fn load_type_hints_file(path: &Path) -> Result<TypeHints, TypeHintLoadError> {
    debug!("Loading type hints from {}", path.display());
    let contents = std::fs::read(path)?;
    let type_hints: TypeHints = serde_json::from_slice(&contents)?;
    Ok(type_hints)
}
