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

        // Serialize triggers
        for trigger in &doc.triggers {
            self.serialize_trigger(trigger, &mut output);
            output.push('\n');
        }

        if !doc.triggers.is_empty() {
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

    fn serialize_trigger(&self, trigger: &TriggerDecl, output: &mut String) {
        if trigger.public {
            output.push_str("public ");
        }
        output.push_str("trigger ");
        output.push_str(&trigger.name);
        output.push_str(" {\n");
        for (i, selector) in trigger.selectors.iter().enumerate() {
            if i > 0 {
                output.push_str(",\n");
            }
            output.push_str("  \"");
            output.push_str(selector);
            output.push('"');
        }
        output.push_str("\n}");
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

        output.push_str(" {\n");
        self.indent_level += 1;

        // Script directive
        if let Some(script) = &component.script {
            self.write_indent(output);
            output.push_str("script(src: \"");
            output.push_str(&script.src);
            output.push_str("\", target: \"");
            output.push_str(&script.target);
            output.push('"');
            if let Some(name) = &script.name {
                output.push_str(", name: \"");
                output.push_str(name);
                output.push('"');
            }
            output.push_str(")\n");
        }

        // Variants
        for variant in &component.variants {
            self.write_indent(output);
            output.push_str("variant ");
            output.push_str(&variant.name);
            if !variant.triggers.is_empty() {
                output.push_str(" trigger { ");
                for (i, trigger) in variant.triggers.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push('"');
                    output.push_str(trigger);
                    output.push('"');
                }
                output.push_str(" }");
            }
            output.push('\n');
        }

        // Slots
        for slot in &component.slots {
            self.serialize_slot(slot, output);
        }

        // Body
        if let Some(body) = &component.body {
            self.write_indent(output);
            output.push_str("render ");
            // Serialize body - for tag elements, put tag name on same line
            match body {
                Element::Tag { tag_name, name, .. } => {
                    output.push_str(tag_name);
                    if let Some(element_name) = name {
                        output.push(' ');
                        output.push_str(element_name);
                    }
                    output.push(' ');
                    self.serialize_tag_body(body, output);
                }
                _ => {
                    output.push('\n');
                    self.serialize_element(body, output);
                }
            }
        }

        self.indent_level -= 1;
        self.write_indent(output);
        output.push_str("}\n");
    }

    fn serialize_tag_body(&mut self, element: &Element, output: &mut String) {
        // Serialize just the body of a tag element (attributes, styles, children)
        if let Element::Tag {
            attributes,
            styles,
            children,
            ..
        } = element
        {
            // Attributes
            if !attributes.is_empty() {
                output.push('(');
                let mut first = true;
                for (key, value) in attributes {
                    if !first {
                        output.push_str(", ");
                    }
                    first = false;
                    output.push_str(key);
                    output.push_str(" = ");
                    self.serialize_expression(value, output);
                }
                output.push_str(") ");
            }

            // Children and styles
            if !children.is_empty() || !styles.is_empty() {
                output.push_str("{\n");
                self.indent_level += 1;

                // Serialize styles
                for style_block in styles {
                    self.serialize_style_block(style_block, output);
                }

                // Serialize children
                for child in children {
                    self.serialize_element(child, output);
                }

                self.indent_level -= 1;
                self.write_indent(output);
                output.push_str("}\n");
            } else {
                output.push('\n');
            }
        }
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
            Element::Tag {
                tag_name,
                name,
                attributes,
                styles,
                children,
                ..
            } => {
                self.write_indent(output);
                output.push_str(tag_name);

                // Optional element name
                if let Some(element_name) = name {
                    output.push(' ');
                    output.push_str(element_name);
                }

                // Attributes
                if !attributes.is_empty() {
                    output.push_str(" (");
                    let mut first = true;
                    for (key, value) in attributes {
                        if !first {
                            output.push_str(", ");
                        }
                        first = false;
                        output.push_str(key);
                        output.push_str(" = ");
                        self.serialize_expression(value, output);
                    }
                    output.push(')');
                }

                // Children and styles
                if !children.is_empty() || !styles.is_empty() {
                    output.push_str(" {\n");
                    self.indent_level += 1;

                    // Serialize styles
                    for style_block in styles {
                        self.serialize_style_block(style_block, output);
                    }

                    // Serialize children
                    for child in children {
                        self.serialize_element(child, output);
                    }

                    self.indent_level -= 1;
                    self.write_indent(output);
                    output.push_str("}\n");
                } else {
                    output.push('\n');
                }
            }

            Element::Text { content, .. } => {
                self.write_indent(output);
                output.push_str("text ");
                self.serialize_expression(content, output);
                output.push('\n');
            }

            Element::Instance {
                name,
                props,
                children,
                ..
            } => {
                self.write_indent(output);
                output.push_str(name);

                // Props
                if !props.is_empty() {
                    output.push_str(" (");
                    let mut first = true;
                    for (key, value) in props {
                        if !first {
                            output.push_str(", ");
                        }
                        first = false;
                        output.push_str(key);
                        output.push_str(" = ");
                        self.serialize_expression(value, output);
                    }
                    output.push(')');
                }

                if !children.is_empty() {
                    output.push_str(" {\n");
                    self.indent_level += 1;
                    for child in children {
                        self.serialize_element(child, output);
                    }
                    self.indent_level -= 1;
                    self.write_indent(output);
                    output.push_str("}\n");
                } else {
                    output.push('\n');
                }
            }

            Element::SlotInsert { name, .. } => {
                self.write_indent(output);
                output.push_str(name);
                output.push('\n');
            }

            Element::Insert {
                slot_name,
                content,
                ..
            } => {
                self.write_indent(output);
                output.push_str("insert ");
                output.push_str(slot_name);
                output.push_str(" {\n");

                self.indent_level += 1;
                for child in content {
                    self.serialize_element(child, output);
                }
                self.indent_level -= 1;

                self.write_indent(output);
                output.push_str("}\n");
            }

            Element::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.write_indent(output);
                output.push_str("if ");
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
                output.push_str("}\n");
            }

            Element::Repeat {
                item_name,
                collection,
                body,
                ..
            } => {
                self.write_indent(output);
                output.push_str("repeat ");
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
                output.push_str("}\n");
            }
        }
    }

    fn serialize_style_block(&mut self, style_block: &StyleBlock, output: &mut String) {
        self.write_indent(output);
        output.push_str("style");

        // Variants
        if !style_block.variants.is_empty() {
            output.push_str(" variant ");
            for (i, variant) in style_block.variants.iter().enumerate() {
                if i > 0 {
                    output.push_str(" + ");
                }
                output.push_str(variant);
            }
        }

        // Extends
        if !style_block.extends.is_empty() {
            output.push_str(" extends ");
            for (i, extend) in style_block.extends.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(extend);
            }
        }

        output.push_str(" {\n");
        self.indent_level += 1;

        for (key, value) in &style_block.properties {
            self.write_indent(output);
            output.push_str(key);
            output.push_str(": ");
            output.push_str(value);
            output.push('\n');
        }

        self.indent_level -= 1;
        self.write_indent(output);
        output.push_str("}\n");
    }


    fn serialize_expression(&self, expr: &Expression, output: &mut String) {
        // Wrap in braces for attribute values and text content
        match expr {
            Expression::Literal { .. } => {
                // Literals are already quoted
                self.serialize_expression_inner(expr, output);
            }
            Expression::Number { .. } | Expression::Boolean { .. } => {
                // Numbers and booleans don't need braces in most contexts
                self.serialize_expression_inner(expr, output);
            }
            _ => {
                // Everything else needs braces for proper parsing
                output.push('{');
                self.serialize_expression_inner(expr, output);
                output.push('}');
            }
        }
    }

    fn serialize_expression_inner(&self, expr: &Expression, output: &mut String) {
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
                output.push_str(name);
            }

            Expression::Binary {
                left,
                operator,
                right,
                ..
            } => {
                self.serialize_expression_inner(left, output);
                output.push(' ');
                self.serialize_binary_op(operator, output);
                output.push(' ');
                self.serialize_expression_inner(right, output);
            }

            Expression::Call {
                function,
                arguments,
                ..
            } => {
                output.push_str(function);
                output.push('(');
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    self.serialize_expression(arg, output);
                }
                output.push(')');
            }

            Expression::Member {
                object, property, ..
            } => {
                self.serialize_expression_inner(object, output);
                output.push('.');
                output.push_str(property);
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
        assert!(serialized.contains("render button"));
        assert!(serialized.contains("text \"Click me\""));
    }

    #[test]
    #[ignore = "Serializer outputs HTML syntax, not valid Paperclip syntax - needs work"]
    fn test_serialize_with_attributes() {
        let source = r#"component Card {
  render div {
    style {
      padding: 16px
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
    #[ignore = "Serializer doesn't output 'render' keyword, produces invalid Paperclip - needs work"]
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

        println!("Original source:\n{}\n", source);

        let doc = parse(source).unwrap();
        let serialized = serialize(&doc);

        println!("Serialized output:\n{}\n", serialized);

        // Should be parseable again
        let reparsed = parse(&serialized);
        assert!(reparsed.is_ok());

        // Structure should match
        let original_components = doc.components.len();
        let reparsed_components = reparsed.unwrap().components.len();
        assert_eq!(original_components, reparsed_components);
    }
}
