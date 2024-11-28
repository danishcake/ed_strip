use std::path::PathBuf;

use glob::{GlobError, PatternError};
use rayon::ThreadPoolBuildError;
use thiserror::Error;
use tree_sitter::LanguageError;

use crate::type_hints::TypeHintLoadError;

/// An error raised during the stripping process
#[derive(Error, Debug)]
pub enum StrippingError {
    #[error("multiple type hints found: {path}")]
    MultipleTypeHintsFound { path: PathBuf },

    #[error("no stripper found")]
    NoStripperFound { path: PathBuf },

    #[error("multiple strippers found for '{path}'. Consider adding a type hint similar to {suggestion}")]
    MultipleStrippersFound { path: PathBuf, suggestion: String },

    // Typically this means unable to access the file for permissions reasons
    #[error("glob error: {0}")]
    GlobError(#[from] GlobError),

    #[error("glob pattern error")]
    PatternError(#[from] PatternError),

    // Unlikely to occur, but caused by problems loading a TreeSitter language
    #[error("language error: {0}")]
    LanguageError(#[from] LanguageError),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Path manipulation error: {0}")]
    PathError(#[from] std::path::StripPrefixError),
}

pub type StrippingResult<T> = Result<T, StrippingError>;

/// A unified error type
#[derive(Error, Debug)]
pub enum EdStripError {
    #[error("threadpool initialisation error: {0}")]
    ThreadPoolBuildError(#[from] ThreadPoolBuildError),

    #[error("type hint loading error: {0}")]
    TypeHintLoadError(#[from] TypeHintLoadError),

    #[error("stripping error: {0}")]
    StrippingError(#[from] StrippingError),
}

pub type EdStripResult<T> = Result<T, EdStripError>;
