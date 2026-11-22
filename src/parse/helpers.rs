//! Tree-sitter helper functions (adapted from veriplan).

/// Get text from a tree-sitter node.
///
/// Total: a range outside the source, or one that splits a character, yields
/// None rather than a panic.
pub fn node_text<'a>(node: &tree_sitter::Node, source: &'a [u8]) -> Option<&'a str> {
    let slice = source.get(node.byte_range())?;
    std::str::from_utf8(slice).ok()
}

/// Find child node by kind.
pub fn find_child<'a>(node: &tree_sitter::Node<'a>, kind: &str) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|&child| child.kind() == kind)
}
