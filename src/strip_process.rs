//! This file contains the overall stripping process.
//! There are two public methods:
//! * find_files. This returns a list of files matching a glob pattern
//! * process_file. This strips an individual file, and writes the stripped source back
//!
//! The steps involved in process file are:
//! * identify_language. This finds the most appropriate language to process a file.
//!   This is primarily driven by the file extension, but where ambiguities are present
//!   (e.g. .h files can be stripped by C, C++, Objective C etc), then type hints can be used
//!   to resolve this. Files with non-standard file extensions can be handled similarly.
//! * load_file. Reads the contents of the file and returns it as a UTF-8 encoded string.
//! * strip_file. Having identified the language, the comments are removed from the source
//! * write_file. The stripped file is written back to disk
use std::{fs, path::Path};

use glob::{glob, Paths};
use tree_sitter::Parser as TSParser;

use crate::{
    errors::{StrippingError, StrippingResult},
    languages::{LanguageDefinition, LANGUAGES},
    strip_core::strip_comments,
};

/// Identifies the language
///
/// This iterates through the list of available languages and finds those that can handle
/// a given file extension.
/// In future this will be extended with type hints to allow this process to be overridden.
///
/// # Arguments
/// * `path` - The full path to the file
///
/// # Return
/// On success, a single matching LanguageDefinition
fn identify_language(path: &Path) -> Result<&LanguageDefinition, StrippingError> {
    // Identify the appropriate language
    let matching_languages: Vec<&&LanguageDefinition> = LANGUAGES
        .iter()
        .filter(|language| {
            if let Some(path_extension) = path.extension() {
                if let Some(path_extension) = path_extension.to_str() {
                    return language.file_extensions.contains(path_extension);
                }
            }
            false
        })
        .collect();

    // Ensure only a single item language matches
    let language = match matching_languages.len() {
        0 => {
            return Err(StrippingError::NoStripperFound {
                path: path.to_path_buf(),
            })
        }
        1 => matching_languages[0],
        _ => {
            return Err(StrippingError::MultipleStrippersFound {
                path: path.to_path_buf(),
            })
        }
    };

    Ok(language)
}

/// Loads the file to be stripped
///
/// # Arguments
/// * path - The path to load. This must refer to a UTF-8 encoded string at present
///
/// # Return
/// On success, the contents of the file.
fn load_file(path: &Path) -> Result<String, StrippingError> {
    // TODO: Other encodings
    // TODO: Normalise line endings to \n
    let source = fs::read_to_string(path)?;

    Ok(source)
}

/// Performs the actual stripping for a single file
///
/// # Arguments
/// * language - The language to strip as
/// * source - A string containing the source to strip
///
/// # Return
/// On success, the source code with all comments removed
fn strip_file(language: &LanguageDefinition, source: String) -> Result<String, StrippingError> {
    // Create a parser for the detected language
    let mut parser = TSParser::new();
    parser.set_language(&language.language)?;

    // Parse the source
    let mut tree = parser.parse(source.clone(), None).unwrap();

    // Strip the source
    Ok(strip_comments(&mut tree, &language, &source))
}

/// Writes the stripped source back to disk
///
/// The file will be written to the output dir at the relative path of path to input_dir.
/// For example, if input dir is /a/b/c, the output_dir is /g/h/i, and the file is /a/b/c/d/e/f
/// then /g/h/i/d/e/f will be written.
///
/// # Arguments
/// * path - The path the file was originally read from
/// * input_dir - The directory that was searched to find the input file
/// * output_dir - The directory to write to
/// * source - The stripped source
fn write_file(
    path: &Path,
    input_dir: &Path,
    output_dir: &Path,
    source: String,
) -> Result<(), StrippingError> {
    // Determine output directory by stripping the input directory prefix
    // and appending the output directory prefix
    let input_relative_path = path.strip_prefix(input_dir)?;
    let output_path = output_dir.join(input_relative_path);
    fs::write(output_path, source)?;
    Ok(())
}

/// Finds all jobs in the provided input directory using the glob pattern
///
/// # Arguments
/// * input_dir - The directory to search
/// * glob_pattern - The glob to search with
///
/// # Return
/// On success, an iterable of Path
pub fn find_files(input_dir: &Path, glob_pattern: &str) -> StrippingResult<Paths> {
    Ok(glob(&format!(
        "{}/{}",
        input_dir.to_string_lossy(),
        glob_pattern
    ))?)
}

/// Performs processing for a single file
///
/// Identifies the language of a file, strips comments, and writes it back to disk
///
/// # Arguments
/// * input_dir - The directory to search
/// * output_dir - The directory to write results to
/// * path - The path to a single file to process
pub fn process_file(
    input_dir: &Path,
    output_dir: &Path,
    path: &Path,
) -> Result<(), StrippingError> {
    let language = identify_language(path)?;
    let source = load_file(path)?;
    let stripped_source = strip_file(language, source)?;
    write_file(path, input_dir, output_dir, stripped_source)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GIVEN A path to a Python file
    /// WHEN identify_language is called
    /// THEN the language is identified
    #[test]
    fn identify_language_finds_python() {
        let result = identify_language(Path::new("/tmp/test.py"));
        assert!(matches!(result, Ok(_)));
    }

    /// GIVEN A path to a C/C++/ObjC header
    /// WHEN identify_language is called
    /// THEN an error is returned as the language is ambiguous
    #[test]
    fn identify_language_ambiguity() {
        let result = identify_language(Path::new("/tmp/test.h"));
        assert!(matches!(result, Err(_)));
    }

    /// GIVEN A path to an unknown file type
    /// WHEN identify_language is called
    /// THEN an error is returned as the language is unknown
    #[test]
    fn identify_language_unknown() {
        let result = identify_language(Path::new("/tmp/test.bin"));
        assert!(matches!(result, Err(_)));
    }
}
