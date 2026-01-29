use paperclip_parser::ast::*;

/// Visitor pattern for traversing AST nodes immutably
///
/// This trait provides default implementations that walk the entire tree.
/// Override specific visit_* methods to perform custom actions on nodes.
pub trait Visitor: Sized {
    fn visit_document(&mut self, doc: &Document) {
        walk_document(self, doc);
    }

    fn visit_import(&mut self, _import: &Import) {
        // Leaf node, no children to walk
    }

    fn visit_token_decl(&mut self, _token: &TokenDecl) {
        // Leaf node, no children to walk
    }

    fn visit_trigger_decl(&mut self, _trigger: &TriggerDecl) {
        // Leaf node, no children to walk
    }

    fn visit_style_decl(&mut self, _style: &StyleDecl) {
        // Leaf node, no children to walk
    }

    fn visit_component(&mut self, component: &Component) {
        walk_component(self, component);
    }

    fn visit_element(&mut self, element: &Element) {
        walk_element(self, element);
    }

    fn visit_style_block(&mut self, _style: &StyleBlock) {
        // Leaf node, no children to walk
    }

    fn visit_expression(&mut self, expr: &Expression) {
        walk_expression(self, expr);
    }
}

/// Mutable visitor pattern for transforming AST nodes
///
/// Similar to Visitor, but provides mutable access to nodes.
/// Use this when you need to modify the AST during traversal.
pub trait VisitorMut: Sized {
    fn visit_document_mut(&mut self, doc: &mut Document) {
        walk_document_mut(self, doc);
    }

    fn visit_import_mut(&mut self, _import: &mut Import) {
        // Leaf node, no children to walk
    }

    fn visit_token_decl_mut(&mut self, _token: &mut TokenDecl) {
        // Leaf node, no children to walk
    }

    fn visit_trigger_decl_mut(&mut self, _trigger: &mut TriggerDecl) {
        // Leaf node, no children to walk
    }

    fn visit_style_decl_mut(&mut self, _style: &mut StyleDecl) {
        // Leaf node, no children to walk
    }

    fn visit_component_mut(&mut self, component: &mut Component) {
        walk_component_mut(self, component);
    }

    fn visit_element_mut(&mut self, element: &mut Element) {
        walk_element_mut(self, element);
    }

    fn visit_style_block_mut(&mut self, _style: &mut StyleBlock) {
        // Leaf node, no children to walk
    }

    fn visit_expression_mut(&mut self, expr: &mut Expression) {
        walk_expression_mut(self, expr);
    }
}

// Default walk implementations for immutable visitor

pub fn walk_document<V: Visitor>(visitor: &mut V, doc: &Document) {
    for import in &doc.imports {
        visitor.visit_import(import);
    }
    for token in &doc.tokens {
        visitor.visit_token_decl(token);
    }
    for trigger in &doc.triggers {
        visitor.visit_trigger_decl(trigger);
    }
    for style in &doc.styles {
        visitor.visit_style_decl(style);
    }
    for component in &doc.components {
        visitor.visit_component(component);
    }
}

pub fn walk_component<V: Visitor>(visitor: &mut V, component: &Component) {
    for slot in &component.slots {
        for element in &slot.default_content {
            visitor.visit_element(element);
        }
    }

    if let Some(body) = &component.body {
        visitor.visit_element(body);
    }
}

pub fn walk_element<V: Visitor>(visitor: &mut V, element: &Element) {
    match element {
        Element::Tag {
            attributes,
            styles,
            children,
            ..
        } => {
            for expr in attributes.values() {
                visitor.visit_expression(expr);
            }
            for style in styles {
                visitor.visit_style_block(style);
            }
            for child in children {
                visitor.visit_element(child);
            }
        }
        Element::Text { content, .. } => {
            visitor.visit_expression(content);
        }
        Element::Instance {
            props, children, ..
        } => {
            for expr in props.values() {
                visitor.visit_expression(expr);
            }
            for child in children {
                visitor.visit_element(child);
            }
        }
        Element::Conditional {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visitor.visit_expression(condition);
            for element in then_branch {
                visitor.visit_element(element);
            }
            if let Some(else_elements) = else_branch {
                for element in else_elements {
                    visitor.visit_element(element);
                }
            }
        }
        Element::Repeat {
            collection, body, ..
        } => {
            visitor.visit_expression(collection);
            for element in body {
                visitor.visit_element(element);
            }
        }
        Element::SlotInsert { .. } => {
            // No children to walk
        }
        Element::Insert { content, .. } => {
            for element in content {
                visitor.visit_element(element);
            }
        }
    }
}

pub fn walk_expression<V: Visitor>(visitor: &mut V, expr: &Expression) {
    match expr {
        Expression::Literal { .. }
        | Expression::Number { .. }
        | Expression::Boolean { .. }
        | Expression::Variable { .. } => {
            // Leaf nodes
        }
        Expression::Member { object, .. } => {
            visitor.visit_expression(object);
        }
        Expression::Binary { left, right, .. } => {
            visitor.visit_expression(left);
            visitor.visit_expression(right);
        }
        Expression::Call { arguments, .. } => {
            for arg in arguments {
                visitor.visit_expression(arg);
            }
        }
        Expression::Template { parts, .. } => {
            for part in parts {
                if let TemplatePart::Expression(expr) = part {
                    visitor.visit_expression(expr);
                }
            }
        }
    }
}

// Default walk implementations for mutable visitor

pub fn walk_document_mut<V: VisitorMut>(visitor: &mut V, doc: &mut Document) {
    for import in &mut doc.imports {
        visitor.visit_import_mut(import);
    }
    for token in &mut doc.tokens {
        visitor.visit_token_decl_mut(token);
    }
    for trigger in &mut doc.triggers {
        visitor.visit_trigger_decl_mut(trigger);
    }
    for style in &mut doc.styles {
        visitor.visit_style_decl_mut(style);
    }
    for component in &mut doc.components {
        visitor.visit_component_mut(component);
    }
}

pub fn walk_component_mut<V: VisitorMut>(visitor: &mut V, component: &mut Component) {
    for slot in &mut component.slots {
        for element in &mut slot.default_content {
            visitor.visit_element_mut(element);
        }
    }

    if let Some(body) = &mut component.body {
        visitor.visit_element_mut(body);
    }
}

pub fn walk_element_mut<V: VisitorMut>(visitor: &mut V, element: &mut Element) {
    match element {
        Element::Tag {
            attributes,
            styles,
            children,
            ..
        } => {
            for expr in attributes.values_mut() {
                visitor.visit_expression_mut(expr);
            }
            for style in styles {
                visitor.visit_style_block_mut(style);
            }
            for child in children {
                visitor.visit_element_mut(child);
            }
        }
        Element::Text { content, .. } => {
            visitor.visit_expression_mut(content);
        }
        Element::Instance {
            props, children, ..
        } => {
            for expr in props.values_mut() {
                visitor.visit_expression_mut(expr);
            }
            for child in children {
                visitor.visit_element_mut(child);
            }
        }
        Element::Conditional {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            visitor.visit_expression_mut(condition);
            for element in then_branch {
                visitor.visit_element_mut(element);
            }
            if let Some(else_elements) = else_branch {
                for element in else_elements {
                    visitor.visit_element_mut(element);
                }
            }
        }
        Element::Repeat {
            collection, body, ..
        } => {
            visitor.visit_expression_mut(collection);
            for element in body {
                visitor.visit_element_mut(element);
            }
        }
        Element::SlotInsert { .. } => {
            // No children to walk
        }
        Element::Insert { content, .. } => {
            for element in content {
                visitor.visit_element_mut(element);
            }
        }
    }
}

pub fn walk_expression_mut<V: VisitorMut>(visitor: &mut V, expr: &mut Expression) {
    match expr {
        Expression::Literal { .. }
        | Expression::Number { .. }
        | Expression::Boolean { .. }
        | Expression::Variable { .. } => {
            // Leaf nodes
        }
        Expression::Member { object, .. } => {
            visitor.visit_expression_mut(object);
        }
        Expression::Binary { left, right, .. } => {
            visitor.visit_expression_mut(left);
            visitor.visit_expression_mut(right);
        }
        Expression::Call { arguments, .. } => {
            for arg in arguments {
                visitor.visit_expression_mut(arg);
            }
        }
        Expression::Template { parts, .. } => {
            for part in parts {
                if let TemplatePart::Expression(expr) = part {
                    visitor.visit_expression_mut(expr);
                }
            }
        }
    }
}
