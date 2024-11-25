use tree_sitter::{Tree, TreeCursor};

use crate::languages::LanguageDefinition;

/// Determines if the cursor lies on a comment
fn is_comment(cursor: &TreeCursor<'_>, language_definition: &LanguageDefinition) -> bool {
    language_definition
        .comment_node_types
        .contains(cursor.node().kind())
}

/// Generates a string that can replace the comment
/// This is guaranteed to contain the same number of newlines
fn comment_replacement(cursor: &TreeCursor<'_>, source_code: &str) -> String {
    let node = cursor.node();
    let node_source = &source_code[node.byte_range()];
    let newline_count = node_source.chars().filter(|c| *c == '\n').count();
    "\n".repeat(newline_count)
}

/// Strips comments from the source code
/// Input must contain \n newlines only
pub fn strip_comments(
    tree: &mut Tree,
    language_definition: &LanguageDefinition,
    source_code: &str,
) -> String {
    // First visit child nodes. We only need to visit the first?
    // If no child nodes, visit next siblings
    let mut cursor = tree.walk();
    // As we replace code, the output will gradually get shorter
    let mut truncate_offset = 0usize;
    let mut result: String = source_code.into();

    loop {
        if is_comment(&cursor, language_definition) {
            let replacement = comment_replacement(&cursor, source_code);
            let mut range = cursor.node().byte_range();

            range.start -= truncate_offset;
            range.end -= truncate_offset;
            truncate_offset += range.len();
            truncate_offset -= replacement.len();
            result.replace_range(range, &replacement);
        }
        // Visit children, unless we just nuked the node
        else if cursor.goto_first_child() {
            // Successfully went to child node, continue loop
            continue;
        }

        // Once we've visited all children, visit siblings
        if cursor.goto_next_sibling() {
            // Successfully went to sibling node, continue loop
            continue;
        }

        // Once we've visited all siblings, return to parent.
        // This !goto_next_sibling construct unwinds the stack in a minimal number of steps,
        // avoiding repeatedly printing nodes we've already visited
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                // Reached the root again. Terminate search
                return result;
            }
        }
    }
}
