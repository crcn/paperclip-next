use crate::ast::*;
use std::collections::HashSet;

/// Lossless serializer that preserves original formatting using spans
///
/// This serializer enables perfect roundtrip editing:
/// 1. Parse source → AST with spans
/// 2. Edit AST (change names, reorder, etc.)
/// 3. Serialize → preserve all original whitespace/comments
///
/// Strategy:
/// - Track which spans have been "dirty" (modified)
/// - For clean spans: copy original source verbatim
/// - For dirty spans: re-serialize that node only
/// - Preserve whitespace between nodes
pub struct LosslessSerializer<'a> {
    source: &'a str,
    dirty_spans: HashSet<String>, // Node IDs that were modified
}

impl<'a> LosslessSerializer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            dirty_spans: HashSet::new(),
        }
    }

    /// Mark a node as dirty (modified) by its span ID
    pub fn mark_dirty(&mut self, span_id: &str) {
        self.dirty_spans.insert(span_id.to_string());
    }

    /// Mark multiple nodes as dirty
    pub fn mark_dirty_many(&mut self, span_ids: &[String]) {
        for id in span_ids {
            self.dirty_spans.insert(id.clone());
        }
    }

    /// Serialize document with span-based preservation
    pub fn serialize(&self, doc: &Document) -> String {
        let mut output = String::new();
        let mut last_end = 0;

        // Collect all top-level spans in order
        let mut spans: Vec<(&Span, SerializableNode)> = Vec::new();

        for import in &doc.imports {
            spans.push((&import.span, SerializableNode::Import(import)));
        }
        for token in &doc.tokens {
            spans.push((&token.span, SerializableNode::Token(token)));
        }
        for trigger in &doc.triggers {
            spans.push((&trigger.span, SerializableNode::Trigger(trigger)));
        }
        for style in &doc.styles {
            spans.push((&style.span, SerializableNode::Style(style)));
        }
        for component in &doc.components {
            spans.push((&component.span, SerializableNode::Component(component)));
        }

        // Sort by start position
        spans.sort_by_key(|(span, _)| span.start);

        // Process each node
        for (span, node) in spans {
            // Preserve whitespace/comments before this node
            if span.start > last_end {
                output.push_str(&self.source[last_end..span.start]);
            }

            // Check if this node or any child is dirty
            if self.is_dirty_recursive(span, &node) {
                // Re-serialize this node
                self.serialize_node(&node, &mut output);
            } else {
                // Copy original source verbatim
                output.push_str(&self.source[span.start..span.end]);
            }

            last_end = span.end;
        }

        // Preserve trailing whitespace/comments
        if last_end < self.source.len() {
            output.push_str(&self.source[last_end..]);
        }

        output
    }

    /// Check if a node or any of its children are dirty
    fn is_dirty_recursive(&self, span: &Span, node: &SerializableNode) -> bool {
        if self.dirty_spans.contains(&span.id) {
            return true;
        }

        // Check children recursively
        match node {
            SerializableNode::Component(c) => {
                if let Some(body) = &c.body {
                    return self.is_element_dirty(body);
                }
            }
            SerializableNode::Style(_s) => {
                // Styles are atomic for now
                return false;
            }
            _ => return false,
        }

        false
    }

    fn is_element_dirty(&self, elem: &Element) -> bool {
        if self.dirty_spans.contains(&elem.span().id) {
            return true;
        }

        // Check children based on element type
        match elem {
            Element::Tag { children, .. } => {
                for child in children {
                    if self.is_element_dirty(child) {
                        return true;
                    }
                }
            }
            Element::Instance { children, .. } => {
                for child in children {
                    if self.is_element_dirty(child) {
                        return true;
                    }
                }
            }
            Element::Conditional {
                then_branch,
                else_branch,
                ..
            } => {
                for child in then_branch {
                    if self.is_element_dirty(child) {
                        return true;
                    }
                }
                if let Some(else_br) = else_branch {
                    for child in else_br {
                        if self.is_element_dirty(child) {
                            return true;
                        }
                    }
                }
            }
            Element::Repeat { body, .. } => {
                for child in body {
                    if self.is_element_dirty(child) {
                        return true;
                    }
                }
            }
            Element::Text { .. } | Element::Insert { .. } | Element::SlotInsert { .. } => {
                // No children to check
            }
        }

        false
    }

    /// Re-serialize a node (fallback to regular serializer)
    fn serialize_node(&self, node: &SerializableNode, output: &mut String) {
        match node {
            SerializableNode::Import(i) => {
                output.push_str("import ");
                output.push_str(&i.path);
                if let Some(alias) = &i.alias {
                    output.push_str(" as ");
                    output.push_str(alias);
                }
            }
            SerializableNode::Token(t) => {
                if t.public {
                    output.push_str("public ");
                }
                output.push_str("token ");
                output.push_str(&t.name);
                output.push(' ');
                output.push_str(&t.value);
            }
            SerializableNode::Trigger(t) => {
                if t.public {
                    output.push_str("public ");
                }
                output.push_str("trigger ");
                output.push_str(&t.name);
                output.push_str(" { /* TODO: selectors */ }");
            }
            SerializableNode::Style(s) => {
                if s.public {
                    output.push_str("public ");
                }
                output.push_str("style ");
                output.push_str(&s.name);
                output.push_str(" { /* TODO: properties */ }");
            }
            SerializableNode::Component(c) => {
                // Use the regular serializer for components
                let component_output = crate::serializer::serialize_component(c);
                output.push_str(&component_output);
            }
        }
    }
}

/// Helper enum for tracking node types during serialization
enum SerializableNode<'a> {
    Import(&'a Import),
    Token(&'a TokenDecl),
    Trigger(&'a TriggerDecl),
    Style(&'a StyleDecl),
    Component(&'a Component),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_lossless_roundtrip_no_changes() {
        let source = r#"public component Button {
    render button {
        text "Click me"
    }
}"#;

        let doc = parse(source).expect("Parse failed");
        let serializer = LosslessSerializer::new(source);
        let output = serializer.serialize(&doc);

        assert_eq!(
            output, source,
            "Lossless roundtrip should preserve exact source"
        );
    }

    #[test]
    fn test_lossless_with_comments() {
        let source = r#"// This is a button
public component Button {
    // Render a button element
    render button {
        text "Click me"
    }
}

/* Another comment */
"#;

        let doc = parse(source).expect("Parse failed");
        let serializer = LosslessSerializer::new(source);
        let output = serializer.serialize(&doc);

        assert_eq!(output, source, "Should preserve comments");
    }

    #[test]
    fn test_lossless_with_extra_whitespace() {
        let source = r#"


public component Button {


    render button {
        text "Click me"
    }


}


"#;

        let doc = parse(source).expect("Parse failed");
        let serializer = LosslessSerializer::new(source);
        let output = serializer.serialize(&doc);

        assert_eq!(output, source, "Should preserve extra whitespace");
    }

    #[test]
    fn test_dirty_node_reserialized() {
        let source = r#"public component Button {
    render button {
        text "Click me"
    }
}"#;

        let mut doc = parse(source).expect("Parse failed");

        // Mark component as dirty (simulating a name change)
        let mut serializer = LosslessSerializer::new(source);
        serializer.mark_dirty(&doc.components[0].span.id);

        // Change the component name
        doc.components[0].name = "BigButton".to_string();

        let output = serializer.serialize(&doc);

        println!("Output:\n{}", output);

        assert!(output.contains("BigButton"), "Should contain new name");
        assert!(
            !output.contains("component Button"),
            "Should not contain old component declaration"
        );
        assert!(
            output.contains("component BigButton"),
            "Should contain new component declaration"
        );
    }
}
