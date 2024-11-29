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
use std::{collections::HashSet, fs, path::Path};

use glob::{glob, MatchOptions, Paths};
use log::{debug, warn};
use tree_sitter::Parser as TSParser;

use crate::{
    errors::{StrippingError, StrippingResult},
    languages::{LanguageDefinition, LANGUAGES},
    strip_core::strip_comments,
    type_hints::{TypeHint, TypeHints},
};

/// Identifies the language from a given set of type hints
///
/// This iterates through the list of available hints and finds those that match a file.
/// If multiple hints match a file, the last defined takes effect.
///
/// # Arguments
/// * `path` - The full path to the file
/// * `type_hints` - A type hints structure
///
/// # Return
/// On success, a single matching Option<LanguageDefinition>. If this is None, no language
/// was hinted
fn identify_language_from_hints(
    path: &Path,
    type_hints: &TypeHints,
) -> Result<Option<&'static LanguageDefinition>, StrippingError> {
    debug!(
        "{}: Checking type hints to determine language",
        path.display()
    );

    // Find type hints that match the path
    let matching_hints: Vec<&TypeHint> = type_hints
        .iter()
        .filter(|&th| th.pattern.matches_path(path))
        .collect();

    if matching_hints.len() > 1 {
        warn!(
            "{}: Multiple type hints matched. The lasts will take effect",
            path.display()
        );
    } else if matching_hints.len() == 1 {
        debug!(
            "{}: Type hint matches language {}",
            path.display(),
            matching_hints[0].language
        );
    } else {
        debug!("{}: No type hint found", path.display());
    }

    // Identify the language that corresponds to hint
    let result = matching_hints
        .last()
        .map(|th| {
            LANGUAGES
                .iter()
                .filter_map(|&language| {
                    if language.name == th.language {
                        Some(language)
                    } else {
                        None
                    }
                })
                .nth(0)
        })
        .flatten();

    match (result, matching_hints.len()) {
        (Some(language), 1) => {
            debug!(
                "{}: Type hint selected language {}",
                path.display(),
                language.name
            );
        }
        (Some(language), _) => {
            debug!(
                "{}: Multiple type hints selected language {}",
                path.display(),
                language.name
            );
        }
        (None, 0) => {}
        (None, type_hint_count) => {
            let languages_set: HashSet<String> =
                matching_hints.iter().map(|&f| f.language.clone()).collect();
            let languages_text = languages_set
                .into_iter()
                .collect::<Vec<String>>()
                .join(", ");
            debug!(
                "{}: {} type hints, but language(s) '[{}]' not found. Is it/are they supported language(s)?",
                type_hint_count,
                path.display(),
                languages_text
            );
        }
    };

    Ok(result)
}

/// Identifies the language
///
/// This iterates through the list of available languages and finds those that can handle
/// a given file extension.
/// If no extension is found, it checks a set of globs that can match the filename/path
///
/// # Arguments
/// * `path` - The full path to the file
/// * `type_hints` - A type hints structure
///
/// # Return
/// On success, a single matching LanguageDefinition
fn identify_language_from_filename(
    path: &Path,
) -> Result<&'static LanguageDefinition, StrippingError> {
    debug!(
        "{}: Checking path extension and filenames to determine language",
        path.display()
    );

    // Extract the file extension. If we can't do this, we can't identify any strippers
    let path_extension = path.extension();

    // Convert the extension to a String, which can fail is someone is deliberately passing bad data
    // We'll also make it lowercase so we can do case insensitive file extension checks
    let path_extension = path_extension
        .map(|f| f.to_str())
        .flatten()
        .map(|f| f.to_lowercase());

    match &path_extension {
        Some(path_ext) => {
            debug!("{}: Extension is {}", path.display(), path_ext);
        }
        None => {
            debug!("{}: File has no extension", path.display());
        }
    }

    // Identify the appropriate language
    let matching_languages: Vec<&&LanguageDefinition> = LANGUAGES
        .iter()
        .filter(|&&language| {
            return matches!(&path_extension, Some(path_extension) if language.file_extensions.contains(path_extension.as_str())) ||
                language
                    .path_globs
                    .iter()
                    .any(|path_glob| path_glob.matches_path_with(path, MatchOptions {case_sensitive: false, ..Default::default() }));
        })
        .collect();

    // Ensure only a single item language matches
    let language = match matching_languages.len() {
        0 => {
            warn!(
                "{}: No language could be identified from the file extension/filename",
                path.display()
            );

            return Err(StrippingError::NoStripperFound {
                path: path.to_path_buf(),
            });
        }
        1 => matching_languages[0],
        _ => {
            // This is going to be a fairly common error, so provide a maximally helpful
            // error message
            let matching_languages: Vec<&str> = matching_languages
                .iter()
                .map(|language| language.name)
                .collect();
            let matching_languages = matching_languages.join("/");
            let suggested_pattern = if let Some(path_extension) = &path_extension {
                format!("**/*.{}", path_extension)
            } else {
                path.display().to_string()
            };

            return Err(StrippingError::MultipleStrippersFound {
                path: path.to_path_buf(),
                suggestion: format!(
                    "{{ \"pattern\": \" {}\", \"language\": \"{}\" }}",
                    suggested_pattern, matching_languages
                ),
            });
        }
    };

    debug!("{}: Language detected as {}", path.display(), language.name);

    Ok(language)
}

/// Identifies the language
///
/// This checks the type hints first, and if the file is not hinted, tries to find a stripper based
/// on path extensions.
/// # Arguments
/// * `path` - The full path to the file
/// * `type_hints` - A type hints structure
///
/// # Return
/// On success, a single matching LanguageDefinition
fn identify_language(
    path: &Path,
    type_hints: &TypeHints,
) -> Result<&'static LanguageDefinition, StrippingError> {
    let language = identify_language_from_hints(path, type_hints)?;
    if let Some(language) = language {
        return Ok(language);
    }

    identify_language_from_filename(path)
}

/// Loads the file to be stripped
///
/// # Arguments
/// * `path` - The path to load. This must refer to a UTF-8 encoded string at present
///
/// # Return
/// On success, the contents of the file.
fn load_file(path: &Path) -> Result<String, StrippingError> {
    debug!("{}: Loading", path.display());
    // TODO: Other encodings
    // TODO: Normalise line endings to \n
    let source = fs::read_to_string(path)?;

    debug!("{}: Loaded {} bytes", path.display(), source.len());

    Ok(source)
}

/// Performs the actual stripping for a single file
///
/// # Arguments
/// * `language` - The language to strip as
/// * `source` - A string containing the source to strip
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
/// * `path` - The path the file was originally read from
/// * `input_dir` - The directory that was searched to find the input file
/// * `output_dir` - The directory to write to
/// * `source` - The stripped source
fn write_file(
    path: &Path,
    input_dir: &Path,
    output_dir: &Path,
    source: String,
) -> Result<(), StrippingError> {
    debug!(
        "{}: Writing to output directory {}",
        path.display(),
        output_dir.display()
    );

    // Determine output directory by stripping the input directory prefix
    // and appending the output directory prefix
    let input_relative_path = path.strip_prefix(input_dir)?;
    let output_path = output_dir.join(input_relative_path);
    let output_path_parent = output_path.parent();

    debug!(
        "{}: Writing to path {}",
        path.display(),
        output_path.display()
    );

    // Ensure output directory exists
    if let Some(output_path_parent) = output_path_parent {
        debug!(
            "{}: Creating output directory {}",
            path.display(),
            output_path_parent.display()
        );
        fs::create_dir_all(output_path_parent)?;
    }

    fs::write(output_path, source)?;
    Ok(())
}

/// Finds all jobs in the provided input directory using the glob pattern
///
/// # Arguments
/// * `input_dir` - The directory to search
/// * `glob_pattern` - The glob to search with
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
/// * type_hints - A type hints structure
/// * path - The path to a single file to process
pub fn process_file(
    input_dir: &Path,
    output_dir: &Path,
    type_hints: &TypeHints,
    path: &Path,
) -> Result<(), StrippingError> {
    debug!("{}: Processing", path.display());
    let language = identify_language(path, type_hints)?;
    let source = load_file(path)?;

    debug!("{}: Stripping as {}", path.display(), language.name);
    let stripped_source = strip_file(language, source)?;
    write_file(path, input_dir, output_dir, stripped_source)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use glob::Pattern;

    use crate::type_hints::TypeHint;

    use super::*;

    /// GIVEN A path to a Python file
    /// WHEN identify_language is called
    /// THEN the language is identified
    #[test]
    fn identify_language_finds_python() {
        let result = identify_language(Path::new("/tmp/test.py"), &vec![]);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Python",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }

    /// GIVEN A path to a Python file with unusual capitalisation
    /// WHEN identify_language is called
    /// THEN the language is identified
    #[test]
    fn identify_language_from_extension_is_case_insensitive() {
        let result = identify_language(Path::new("/tmp/test.PY"), &vec![]);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Python",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }

    /// GIVEN A path to an file without an extension, but known language
    /// WHEN identify_language is called
    /// THEN the specified language is returned
    #[test]
    fn identify_language_can_detect_extensionless_files() {
        let type_hints: TypeHints = vec![];

        let result = identify_language(Path::new("/tmp/DockerFile"), &type_hints);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Dockerfile",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }

    /// GIVEN A path to an file without an extension but known language with unusual capitialisation
    /// WHEN identify_language is called
    /// THEN the specified language is returned
    #[test]
    fn identify_language_from_glob_is_case_insensitive() {
        let type_hints: TypeHints = vec![];

        let result = identify_language(Path::new("/tmp/DoCkErFiLe"), &type_hints);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Dockerfile",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }

    /// GIVEN A path to a C/C++/ObjC header
    /// WHEN identify_language is called
    /// THEN an error is returned as the language is ambiguous
    #[test]
    fn identify_language_ambiguity() {
        let result = identify_language(Path::new("/tmp/test.h"), &vec![]);
        assert!(matches!(result, Err(_)));
    }

    /// GIVEN A path to an unknown file type
    /// WHEN identify_language is called
    /// THEN an error is returned as the language is unknown
    #[test]
    fn identify_language_unknown() {
        let result = identify_language(Path::new("/tmp/test.bin"), &vec![]);
        assert!(matches!(result, Err(_)));
    }

    /// GIVEN A path to a known file type
    /// AND a type hint forcing the language definition
    /// WHEN identify_language is called
    /// THEN the alternative language is returned
    #[test]
    fn identify_language_can_use_typehints_for_unknowns() {
        let type_hints: TypeHints = vec![TypeHint {
            pattern: Pattern::from_str("**/*.bin").unwrap().into(),
            language: String::from("Javascript"),
        }];

        let result = identify_language(Path::new("/tmp/test.bin"), &type_hints);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Javascript",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }

    /// GIVEN A path to an unknown file type
    /// AND a type hint forcing the language definition
    /// WHEN identify_language is called
    /// THEN the specified language is returned
    #[test]
    fn identify_language_can_use_typehints_to_override_known() {
        let type_hints: TypeHints = vec![TypeHint {
            pattern: Pattern::from_str("**/*.c").unwrap().into(),
            language: String::from("Javascript"),
        }];

        let result = identify_language(Path::new("/tmp/test.c"), &type_hints);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Javascript",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }

    /// GIVEN A path to an file without an extension
    /// AND a type hint forcing the language definition
    /// WHEN identify_language is called
    /// THEN the specified language is returned
    #[test]
    fn identify_language_can_use_typehints_to_overwrite_extensionless_files() {
        let type_hints: TypeHints = vec![TypeHint {
            pattern: Pattern::from_str("**/Dockerfile").unwrap().into(),
            language: String::from("Javascript"),
        }];

        let result = identify_language(Path::new("/tmp/Dockerfile"), &type_hints);
        assert!(matches!(
            result,
            Ok(LanguageDefinition {
                name: "Javascript",
                file_extensions: _,
                comment_node_types: _,
                path_globs: _,
                language: _
            })
        ));
    }
}
