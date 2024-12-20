use std::{collections::HashSet, str::FromStr};

use glob::Pattern;
use once_cell::sync::Lazy;
use tree_sitter::Language;

/// Defines a supported language
pub struct LanguageDefinition {
    /// The name of the language
    pub name: &'static str,

    /// The file extensions to strip using this stripper
    /// These should all be lower case
    pub file_extensions: Lazy<HashSet<&'static str>>,

    /// The path globs to strip using this stripper
    /// These are more expensive to evaluate, so prefer the file extensions
    pub path_globs: Lazy<Vec<Pattern>>,

    /// The list of tree-sitter nodes that are comments
    pub comment_node_types: Lazy<HashSet<&'static str>>,

    /// The tree-sitter language
    pub language: Lazy<Language>,
}

static RUST: LanguageDefinition = LanguageDefinition {
    name: "Rust",
    comment_node_types: Lazy::new(|| ["line_comment", "block_comment", "doc_comment"].into()),
    file_extensions: Lazy::new(|| ["rs"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(tree_sitter_rust::language),
};

static TYPESCRIPT: LanguageDefinition = LanguageDefinition {
    name: "Typescript",
    // TBD: Suspect html_comment isn't required. It's defined in the node types, but surely a TSX thing?
    comment_node_types: Lazy::new(|| ["comment", "html_comment"].into()),
    file_extensions: Lazy::new(|| ["ts", "mts"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
};

static TYPESCRIPT_REACT: LanguageDefinition = LanguageDefinition {
    name: "Typescript with React",
    comment_node_types: Lazy::new(|| ["comment", "html_comment"].into()),
    file_extensions: Lazy::new(|| ["tsx"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_typescript::LANGUAGE_TSX.into()),
};

static JAVASCRIPT: LanguageDefinition = LanguageDefinition {
    name: "Javascript",
    comment_node_types: Lazy::new(|| ["comment", "html_comment"].into()),
    file_extensions: Lazy::new(|| ["js", "mjs", "cjs", "jsx"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_javascript::LANGUAGE.into()),
};

static GO: LanguageDefinition = LanguageDefinition {
    name: "Go",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["go"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_go::LANGUAGE.into()),
};

static PYTHON: LanguageDefinition = LanguageDefinition {
    name: "Python",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["py"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_python::LANGUAGE.into()),
};

static CPP: LanguageDefinition = LanguageDefinition {
    name: "C++",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["cpp", "cc", "cxx", "h", "hxx", "hpp"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_cpp::LANGUAGE.into()),
};

static C: LanguageDefinition = LanguageDefinition {
    name: "C",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["c", "h"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_c::LANGUAGE.into()),
};

static BASH: LanguageDefinition = LanguageDefinition {
    name: "Bash",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["sh"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_bash::LANGUAGE.into()),
};

static XML: LanguageDefinition = LanguageDefinition {
    name: "XML",
    comment_node_types: Lazy::new(|| ["Comment"].into()),
    file_extensions: Lazy::new(|| ["xml", "vcxproj"].into()),
    path_globs: Lazy::new(|| {
        vec![
            Pattern::from_str("**/*.vcxproj.filters").unwrap(),
            Pattern::from_str("**/*.vcxproj.user").unwrap(),
        ]
    }),
    language: Lazy::new(|| tree_sitter_xml::LANGUAGE_XML.into()),
};

// This library has a [patch] section in cargo.toml
static OBJECTIVE_C: LanguageDefinition = LanguageDefinition {
    name: "Objective-C",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["m", "mm", "h"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(tree_sitter_objc::language),
};

static JAVA: LanguageDefinition = LanguageDefinition {
    name: "Java",
    comment_node_types: Lazy::new(|| ["block_comment", "line_comment"].into()),
    file_extensions: Lazy::new(|| ["java"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_java::LANGUAGE.into()),
};

static HTML: LanguageDefinition = LanguageDefinition {
    name: "HTML",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["htm", "html"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_html::LANGUAGE.into()),
};

// There's also a 'LANGUAGE_PHP_ONLY' mode.
// It's unclear what the difference - I suspect that it only allows PHP,
// as opposed to the more common usage of PHP as a templating solution
static PHP: LanguageDefinition = LanguageDefinition {
    name: "PHP",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["php"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_php::LANGUAGE_PHP.into()),
};

// Unclear what version of Lua this is
static LUA: LanguageDefinition = LanguageDefinition {
    name: "Lua",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["lua"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_lua::LANGUAGE.into()),
};

static SWIFT: LanguageDefinition = LanguageDefinition {
    name: "Swift",
    comment_node_types: Lazy::new(|| ["comment", "multiline_comment"].into()),
    file_extensions: Lazy::new(|| ["swift"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_swift::LANGUAGE.into()),
};

static YAML: LanguageDefinition = LanguageDefinition {
    name: "YAML",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["yaml", "yml"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(tree_sitter_yaml::language),
};

static RUBY: LanguageDefinition = LanguageDefinition {
    name: "Ruby",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["rb"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_ruby::LANGUAGE.into()),
};

static TOML: LanguageDefinition = LanguageDefinition {
    name: "TOML",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["toml"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_toml::language()),
};

static KOTLIN: LanguageDefinition = LanguageDefinition {
    name: "Kotlin",
    comment_node_types: Lazy::new(|| ["line_comment", "multiline_comment"].into()),
    file_extensions: Lazy::new(|| ["kt", "kts"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(tree_sitter_kotlin::language),
};

static PROTO: LanguageDefinition = LanguageDefinition {
    name: "Protobuf",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["pb", "proto"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_proto::LANGUAGE.into()),
};

static C_SHARP: LanguageDefinition = LanguageDefinition {
    name: "C#",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["cs"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_c_sharp::LANGUAGE.into()),
};

static POWERSHELL: LanguageDefinition = LanguageDefinition {
    name: "Powershell",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["ps1"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(tree_sitter_powershell::language),
};

static DOCKERFILE: LanguageDefinition = LanguageDefinition {
    name: "Dockerfile",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["dockerfile"].into()),
    path_globs: Lazy::new(|| {
        vec![
            Pattern::from_str("**/DockerFile").unwrap(),
            Pattern::from_str("**/DockerFile.*").unwrap(),
        ]
    }),
    language: Lazy::new(tree_sitter_dockerfile::language),
};

static CSS: LanguageDefinition = LanguageDefinition {
    name: "CSS",
    // There's also a js_comment, but that's not valid in CSS. Odd!
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["css"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_css::LANGUAGE.into()),
};

static CMAKE: LanguageDefinition = LanguageDefinition {
    name: "CMake",
    comment_node_types: Lazy::new(|| ["line_comment", "bracket_comment"].into()),
    file_extensions: Lazy::new(|| [].into()),
    path_globs: Lazy::new(|| vec![Pattern::from_str("**/CMakeLists.txt").unwrap()]),
    language: Lazy::new(tree_sitter_cmake::language),
};

static HCL: LanguageDefinition = LanguageDefinition {
    name: "HCL",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["tf"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(tree_sitter_hcl::language),
};

static MAKE: LanguageDefinition = LanguageDefinition {
    name: "Make",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["mk"].into()),
    path_globs: Lazy::new(|| vec![Pattern::from_str("makefile").unwrap()]),
    language: Lazy::new(tree_sitter_make::language),
};

static INI: LanguageDefinition = LanguageDefinition {
    name: "Ini",
    comment_node_types: Lazy::new(|| ["comment"].into()),
    file_extensions: Lazy::new(|| ["ini"].into()),
    path_globs: Lazy::new(|| vec![]),
    language: Lazy::new(|| tree_sitter_ini::LANGUAGE.into()),
};

// All supported languages
pub static LANGUAGES: [&LanguageDefinition; 29] = [
    &RUST,
    &TYPESCRIPT,
    &TYPESCRIPT_REACT,
    &JAVASCRIPT,
    &GO,
    &PYTHON,
    &CPP,
    &C,
    &BASH,
    &XML,
    &OBJECTIVE_C,
    &JAVA,
    &HTML,
    &PHP,
    &LUA,
    &SWIFT,
    &YAML,
    &RUBY,
    &KOTLIN,
    &PROTO,
    &TOML,
    &C_SHARP,
    &POWERSHELL,
    &DOCKERFILE,
    &CSS,
    &CMAKE,
    &HCL,
    &MAKE,
    &INI,
];
