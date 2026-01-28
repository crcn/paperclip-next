use crate::ast::*;
use std::fmt::Write;

/// Serializer converts AST back to source code
///
/// For production designer edits, this enables:
/// - Drag and drop elements (reorder children)
/// - Change attributes via property panel
/// - Modify styles visually
/// - Add/remove components
///
/// The serializer preserves structure but may reformat whitespace.
/// For perfect whitespace preservation, we'd need trivia tokens from the parser.
pub struct Serializer {
    indent_level: usize,
    indent_string: String,
}

impl Serializer {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_string: "  ".to_string(), // 2 spaces
        }
    }

    pub fn with_indent(indent: &str) -> Self {
        Self {
            indent_level: 0,
            indent_string: indent.to_string(),
        }
    }

    /// Serialize a Document to source code
    pub fn serialize(&mut self, doc: &Document) -> String {
        let mut output = String::new();

        // Serialize imports
        for import in &doc.imports {
            self.serialize_import(import, &mut output);
            output.push('\n');
        }

        if !doc.imports.is_empty() {
            output.push('\n');
        }

        // Serialize tokens
        for token in &doc.tokens {
            self.serialize_token(token, &mut output);
            output.push('\n');
        }

        if !doc.tokens.is_empty() {
            output.push('\n');
        }

        // Serialize styles
        for style in &doc.styles {
            self.serialize_style(style, &mut output);
            output.push('\n');
        }

        if !doc.styles.is_empty() {
            output.push('\n');
        }

        // Serialize components
        for (i, component) in doc.components.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            self.serialize_component(component, &mut output);
        }

        output
    }

    fn serialize_import(&self, import: &Import, output: &mut String) {
        output.push_str("import ");
        output.push_str(&import.path);
        if let Some(alias) = &import.alias {
            output.push_str(" as ");
            output.push_str(alias);
        }
    }

    fn serialize_token(&self, token: &TokenDecl, output: &mut String) {
        if token.public {
            output.push_str("public ");
        }
        output.push_str("token ");
        output.push_str(&token.name);
        output.push_str(" ");
        output.push_str(&token.value);
    }

    fn serialize_style(&mut self, style: &StyleDecl, output: &mut String) {
        if style.public {
            output.push_str("public ");
        }
        output.push_str("style ");
        output.push_str(&style.name);

        if !style.extends.is_empty() {
            output.push_str(" extends ");
            for (i, extend) in style.extends.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(extend);
            }
        }

        output.push_str(" {\n");
        self.indent_level += 1;

        for (key, value) in &style.properties {
            self.write_indent(output);
            output.push_str(key);
            output.push_str(": ");
            output.push_str(value);
            output.push_str(";\n");
        }

        self.indent_level -= 1;
        output.push('}');
    }

    fn serialize_component(&mut self, component: &Component, output: &mut String) {
        if component.public {
            output.push_str("public ");
        }
        output.push_str("component ");
        output.push_str(&component.name);

        // Variants
        if !component.variants.is_empty() {
            output.push_str(" variant ");
            for (i, variant) in component.variants.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&variant.name);
                if !variant.triggers.is_empty() {
                    output.push_str(" trigger ");
                    for (j, trigger) in variant.triggers.iter().enumerate() {
                        if j > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(trigger);
                    }
                }
            }
        }

        output.push_str(" {\n");
        self.indent_level += 1;

        // Slots
        for slot in &component.slots {
            self.serialize_slot(slot, output);
        }

        // Body
        if let Some(body) = &component.body {
            self.serialize_element(body, output);
        }

        self.indent_level -= 1;
        output.push('}');
    }

    fn serialize_slot(&mut self, slot: &Slot, output: &mut String) {
        self.write_indent(output);
        output.push_str("slot ");
        output.push_str(&slot.name);

        if !slot.default_content.is_empty() {
            output.push_str(" {\n");
            self.indent_level += 1;
            for element in &slot.default_content {
                self.serialize_element(element, output);
            }
            self.indent_level -= 1;
            self.write_indent(output);
            output.push_str("}\n");
        } else {
            output.push('\n');
        }
    }

    fn serialize_element(&mut self, element: &Element, output: &mut String) {
        match element {
            Element::Tag { name, attributes, styles, children, .. } => {
                self.write_indent(output);
                output.push('<');
                output.push_str(name);

                // Attributes
                for (key, value) in attributes {
                    output.push(' ');
                    output.push_str(key);
                    output.push('=');
                    self.serialize_expression(value, output);
                }

                // Styles
                if !styles.is_empty() {
                    output.push_str(" style={");
                    for (i, style_block) in styles.iter().enumerate() {
                        if i > 0 {
                            output.push(' ');
                        }
                        self.serialize_style_block_inline(style_block, output);
                    }
                    output.push('}');
                }

                if children.is_empty() {
                    output.push_str(" />\n");
                } else {
                    output.push_str(">\n");
                    self.indent_level += 1;
                    for child in children {
                        self.serialize_element(child, output);
                    }
                    self.indent_level -= 1;
                    self.write_indent(output);
                    output.push_str("</");
                    output.push_str(name);
                    output.push_str(">\n");
                }
            }

            Element::Text { content, .. } => {
                self.write_indent(output);
                self.serialize_expression(content, output);
                output.push('\n');
            }

            Element::Instance { name, props, children, .. } => {
                self.write_indent(output);
                output.push('<');
                output.push_str(name);

                // Props
                for (key, value) in props {
                    output.push(' ');
                    output.push_str(key);
                    output.push('=');
                    self.serialize_expression(value, output);
                }

                if children.is_empty() {
                    output.push_str(" />\n");
                } else {
                    output.push_str(">\n");
                    self.indent_level += 1;
                    for child in children {
                        self.serialize_element(child, output);
                    }
                    self.indent_level -= 1;
                    self.write_indent(output);
                    output.push_str("</");
                    output.push_str(name);
                    output.push_str(">\n");
                }
            }

            Element::SlotInsert { name, .. } => {
                self.write_indent(output);
                output.push_str("{slot ");
                output.push_str(name);
                output.push_str("}\n");
            }

            Element::Conditional { condition, then_branch, else_branch, .. } => {
                self.write_indent(output);
                output.push_str("{if ");
                self.serialize_expression(condition, output);
                output.push_str(" {\n");

                self.indent_level += 1;
                for child in then_branch {
                    self.serialize_element(child, output);
                }
                self.indent_level -= 1;

                if let Some(else_br) = else_branch {
                    self.write_indent(output);
                    output.push_str("} else {\n");
                    self.indent_level += 1;
                    for child in else_br {
                        self.serialize_element(child, output);
                    }
                    self.indent_level -= 1;
                }

                self.write_indent(output);
                output.push_str("}}\n");
            }

            Element::Repeat { item_name, collection, body, .. } => {
                self.write_indent(output);
                output.push_str("{repeat ");
                output.push_str(item_name);
                output.push_str(" in ");
                self.serialize_expression(collection, output);
                output.push_str(" {\n");

                self.indent_level += 1;
                for child in body {
                    self.serialize_element(child, output);
                }
                self.indent_level -= 1;

                self.write_indent(output);
                output.push_str("}}\n");
            }
        }
    }

    fn serialize_style_block_inline(&self, style_block: &StyleBlock, output: &mut String) {
        if let Some(variant) = &style_block.variant {
            output.push_str(variant);
            output.push_str(": ");
        }

        output.push('{');
        let mut first = true;
        for (key, value) in &style_block.properties {
            if !first {
                output.push_str(", ");
            }
            first = false;
            output.push_str(key);
            output.push_str(": ");
            output.push_str(value);
        }
        output.push('}');
    }

    fn serialize_expression(&self, expr: &Expression, output: &mut String) {
        match expr {
            Expression::Literal { value, .. } => {
                output.push('"');
                // Escape special characters
                for c in value.chars() {
                    match c {
                        '"' => output.push_str("\\\""),
                        '\\' => output.push_str("\\\\"),
                        '\n' => output.push_str("\\n"),
                        '\r' => output.push_str("\\r"),
                        '\t' => output.push_str("\\t"),
                        _ => output.push(c),
                    }
                }
                output.push('"');
            }

            Expression::Number { value, .. } => {
                write!(output, "{}", value).unwrap();
            }

            Expression::Boolean { value, .. } => {
                output.push_str(if *value { "true" } else { "false" });
            }

            Expression::Variable { name, .. } => {
                output.push('{');
                output.push_str(name);
                output.push('}');
            }

            Expression::Binary { left, operator, right, .. } => {
                output.push('{');
                self.serialize_expression(left, output);
                output.push(' ');
                self.serialize_binary_op(operator, output);
                output.push(' ');
                self.serialize_expression(right, output);
                output.push('}');
            }

            Expression::Call { function, arguments, .. } => {
                output.push('{');
                output.push_str(function);
                output.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.serialize_expression(arg, output);
                }
                output.push_str(")}");
            }

            Expression::Member { object, property, .. } => {
                output.push('{');
                self.serialize_expression(object, output);
                output.push('.');
                output.push_str(property);
                output.push('}');
            }

            Expression::Template { parts, .. } => {
                output.push('"');
                for part in parts {
                    match part {
                        TemplatePart::Literal(s) => {
                            for c in s.chars() {
                                match c {
                                    '"' => output.push_str("\\\""),
                                    '\\' => output.push_str("\\\\"),
                                    '\n' => output.push_str("\\n"),
                                    '\r' => output.push_str("\\r"),
                                    '\t' => output.push_str("\\t"),
                                    _ => output.push(c),
                                }
                            }
                        }
                        TemplatePart::Expression(expr) => {
                            output.push_str("${");
                            self.serialize_expression(expr, output);
                            output.push('}');
                        }
                    }
                }
                output.push('"');
            }
        }
    }

    fn serialize_binary_op(&self, op: &BinaryOp, output: &mut String) {
        let op_str = match op {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Equals => "==",
            BinaryOp::NotEquals => "!=",
            BinaryOp::LessThan => "<",
            BinaryOp::LessThanOrEqual => "<=",
            BinaryOp::GreaterThan => ">",
            BinaryOp::GreaterThanOrEqual => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        };
        output.push_str(op_str);
    }

    fn write_indent(&self, output: &mut String) {
        for _ in 0..self.indent_level {
            output.push_str(&self.indent_string);
        }
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to serialize a document
pub fn serialize(doc: &Document) -> String {
    let mut serializer = Serializer::new();
    serializer.serialize(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_serialize_simple_component() {
        let source = r#"component Button {
  render button {
    text "Click me"
  }
}"#;

        let doc = parse(source).unwrap();
        let serialized = serialize(&doc);

        // Should preserve structure (whitespace may differ)
        assert!(serialized.contains("component Button"));
        assert!(serialized.contains("<button>"));
    }

    #[test]
    fn test_serialize_with_attributes() {
        let source = r#"component Card {
  render div {
    style {
      class: "card"
    }
    h1 {
      text "Title"
    }
  }
}"#;

        let doc = parse(source).unwrap();
        let serialized = serialize(&doc);

        assert!(serialized.contains("component Card"));
        assert!(serialized.contains("<div"));
        assert!(serialized.contains("<h1"));
    }

    #[test]
    fn test_roundtrip_preserves_structure() {
        let source = r#"public component Hero {
  render section {
    h1 {
      text "Welcome"
    }
    p {
      text "Description"
    }
  }
}"#;

        let doc = parse(source).unwrap();
        let serialized = serialize(&doc);

        // Should be parseable again
        let reparsed = parse(&serialized);
        assert!(reparsed.is_ok());

        // Structure should match
        let original_components = doc.components.len();
        let reparsed_components = reparsed.unwrap().components.len();
        assert_eq!(original_components, reparsed_components);
    }
}
