/// End-to-End Typing Simulation Tests
///
/// These tests simulate the FULL workflow of a developer typing in VSCode:
/// 1. CRDT document receives character-by-character edits
/// 2. Content is parsed
/// 3. AST is evaluated to VDOM
/// 4. VDOM is diffed against previous state
/// 5. Patches are generated and returned
///
/// This tests the complete pipeline from edit to rendered output.

use crate::crdt::CrdtDocument;
use crate::state::WorkspaceState;
use paperclip_evaluator::{VNode, VDocPatch, diff_vdocument};
use paperclip_evaluator::vdom_differ::proto::patches::v_doc_patch::PatchType;
use std::path::PathBuf;

/// Test harness that simulates the full VSCode -> Server -> Designer pipeline
struct TypingSimulator {
    /// CRDT document (simulates VSCode editor state)
    crdt: CrdtDocument,
    /// Workspace state (server-side state)
    state: WorkspaceState,
    /// File path
    file_path: PathBuf,
    /// Project root
    project_root: PathBuf,
    /// History of patches generated
    patch_history: Vec<PatchResult>,
    /// Current cursor position
    cursor: u32,
}

#[derive(Debug, Clone)]
struct PatchResult {
    /// Whether parsing succeeded
    parse_ok: bool,
    /// Number of patches generated (0 if parse failed)
    patch_count: usize,
    /// Node count in VDOM (from last valid state)
    node_count: usize,
    /// Whether first node has frame
    has_frame: bool,
    /// The current text content
    content: String,
    /// Error message if any
    error: Option<String>,
}

impl TypingSimulator {
    fn new() -> Self {
        Self {
            crdt: CrdtDocument::new(),
            state: WorkspaceState::new(),
            file_path: PathBuf::from("/test/test.pc"),
            project_root: PathBuf::from("/test"),
            patch_history: Vec::new(),
            cursor: 0,
        }
    }

    /// Start with initial content
    fn with_content(content: &str) -> Self {
        let mut sim = Self::new();
        sim.crdt = CrdtDocument::with_content(content);
        sim.cursor = content.len() as u32;
        // Process initial content
        sim.process();
        sim
    }

    /// Type a single character at cursor position
    fn type_char(&mut self, ch: char) -> &PatchResult {
        self.crdt.insert(self.cursor, &ch.to_string());
        self.cursor += 1;
        self.process()
    }

    /// Type a string at cursor position
    fn type_str(&mut self, s: &str) -> &PatchResult {
        self.crdt.insert(self.cursor, s);
        self.cursor += s.len() as u32;
        self.process()
    }

    /// Delete characters backward (like backspace)
    fn backspace(&mut self, count: u32) -> &PatchResult {
        let delete_start = self.cursor.saturating_sub(count);
        let delete_len = self.cursor - delete_start;
        if delete_len > 0 {
            self.crdt.delete(delete_start, delete_len);
            self.cursor = delete_start;
        }
        self.process()
    }

    /// Delete characters forward (like delete key)
    fn delete_forward(&mut self, count: u32) -> &PatchResult {
        let text_len = self.crdt.get_text().len() as u32;
        let delete_len = count.min(text_len - self.cursor);
        if delete_len > 0 {
            self.crdt.delete(self.cursor, delete_len);
        }
        self.process()
    }

    /// Move cursor to position
    fn move_cursor(&mut self, pos: u32) {
        self.cursor = pos;
    }

    /// Move cursor to end
    fn move_to_end(&mut self) {
        self.cursor = self.crdt.get_text().len() as u32;
    }

    /// Replace content in range (like paste over selection)
    fn replace(&mut self, start: u32, end: u32, replacement: &str) -> &PatchResult {
        self.crdt.edit_range(start, end, replacement);
        self.cursor = start + replacement.len() as u32;
        self.process()
    }

    /// Set entire content (like open file)
    fn set_content(&mut self, content: &str) -> &PatchResult {
        // Clear and set new content
        let current_len = self.crdt.get_text().len() as u32;
        self.crdt.edit_range(0, current_len, content);
        self.cursor = content.len() as u32;
        self.process()
    }

    /// Process current CRDT content through the full pipeline
    fn process(&mut self) -> &PatchResult {
        let content = self.crdt.get_text();

        let result = self.state.update_file(
            self.file_path.clone(),
            content.clone(),
            &self.project_root,
        );

        let patch_result = match result {
            Ok(patches) => {
                let file_state = self.state.get_file(&self.file_path)
                    .expect("File should be cached after successful update");

                let has_frame = file_state.vdom.nodes.first()
                    .map(|n| {
                        if let VNode::Element { attributes, .. } = n {
                            attributes.contains_key("data-frame-x")
                        } else {
                            false
                        }
                    })
                    .unwrap_or(false);

                PatchResult {
                    parse_ok: true,
                    patch_count: patches.len(),
                    node_count: file_state.vdom.nodes.len(),
                    has_frame,
                    content,
                    error: None,
                }
            }
            Err(e) => {
                // Get last valid state if available
                let (node_count, has_frame) = self.state.get_file(&self.file_path)
                    .map(|f| {
                        let has_frame = f.vdom.nodes.first()
                            .map(|n| {
                                if let VNode::Element { attributes, .. } = n {
                                    attributes.contains_key("data-frame-x")
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);
                        (f.vdom.nodes.len(), has_frame)
                    })
                    .unwrap_or((0, false));

                PatchResult {
                    parse_ok: false,
                    patch_count: 0,
                    node_count,
                    has_frame,
                    content,
                    error: Some(format!("{:?}", e)),
                }
            }
        };

        self.patch_history.push(patch_result);
        self.patch_history.last().unwrap()
    }

    /// Get current content
    fn content(&self) -> String {
        self.crdt.get_text()
    }

    /// Get the last patch result
    fn last_result(&self) -> Option<&PatchResult> {
        self.patch_history.last()
    }

    /// Get number of nodes in current valid VDOM
    fn node_count(&self) -> usize {
        self.state.get_file(&self.file_path)
            .map(|f| f.vdom.nodes.len())
            .unwrap_or(0)
    }

    /// Check if first node has frame
    fn has_frame(&self) -> bool {
        self.state.get_file(&self.file_path)
            .and_then(|f| f.vdom.nodes.first())
            .map(|n| {
                if let VNode::Element { attributes, .. } = n {
                    attributes.contains_key("data-frame-x")
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    /// Get frame attributes from first node
    fn get_frame(&self) -> Option<(f64, f64, Option<f64>, Option<f64>)> {
        self.state.get_file(&self.file_path)
            .and_then(|f| f.vdom.nodes.first())
            .and_then(|n| {
                if let VNode::Element { attributes, .. } = n {
                    let x = attributes.get("data-frame-x")?.parse().ok()?;
                    let y = attributes.get("data-frame-y")?.parse().ok()?;
                    let w = attributes.get("data-frame-width").and_then(|v| v.parse().ok());
                    let h = attributes.get("data-frame-height").and_then(|v| v.parse().ok());
                    Some((x, y, w, h))
                } else {
                    None
                }
            })
    }

    /// Count how many parse errors occurred
    fn error_count(&self) -> usize {
        self.patch_history.iter().filter(|r| !r.parse_ok).count()
    }

    /// Count how many successful parses occurred
    fn success_count(&self) -> usize {
        self.patch_history.iter().filter(|r| r.parse_ok).count()
    }
}

#[cfg(test)]
mod e2e_typing_tests {
    use super::*;

    // =========================================================================
    // SECTION 1: Basic Typing Scenarios
    // =========================================================================

    #[test]
    fn test_type_simple_text_node() {
        let mut sim = TypingSimulator::new();

        // Type: text "hello"
        for ch in "text \"hello\"".chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok, "Final result should parse");
        assert_eq!(sim.node_count(), 1, "Should have 1 node");
    }

    #[test]
    fn test_type_div_with_text() {
        let mut sim = TypingSimulator::new();

        // Type: div { text "hello" }
        for ch in "div { text \"hello\" }".chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 1);
    }

    #[test]
    fn test_type_component_character_by_character() {
        let mut sim = TypingSimulator::new();

        let component = r#"component Button {
    render button {
        text "Click"
    }
}"#;

        let mut valid_count = 0;
        let mut error_count = 0;

        for ch in component.chars() {
            let result = sim.type_char(ch);
            if result.parse_ok {
                valid_count += 1;
            } else {
                error_count += 1;
            }
        }

        println!("Typed {} chars: {} valid, {} errors",
            component.len(), valid_count, error_count);

        // Final state should be valid
        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 1);
    }

    // =========================================================================
    // SECTION 2: Frame Annotation Typing
    // =========================================================================

    #[test]
    fn test_type_component_with_frame() {
        let mut sim = TypingSimulator::new();

        let component = r#"/**
 * @frame(x: 100, y: 200, width: 400, height: 300)
 */
component Card {
    render div { text "card" }
}"#;

        for ch in component.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.has_frame(), "Should have frame after typing");

        let frame = sim.get_frame().expect("Should have frame");
        assert_eq!(frame.0, 100.0, "x should be 100");
        assert_eq!(frame.1, 200.0, "y should be 200");
        assert_eq!(frame.2, Some(400.0), "width should be 400");
        assert_eq!(frame.3, Some(300.0), "height should be 300");
    }

    #[test]
    fn test_type_render_with_frame() {
        let mut sim = TypingSimulator::new();

        let render = r#"/**
 * @frame(x: 0, y: 0, width: 800, height: 600)
 */
div {
    text "standalone"
}"#;

        for ch in render.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.has_frame(), "Render should have frame");
    }

    #[test]
    fn test_type_multiple_frames() {
        let mut sim = TypingSimulator::new();

        // Type component first
        let component = r#"/**
 * @frame(x: 0, y: 0, width: 400, height: 300)
 */
component First {
    render div { text "first" }
}

"#;

        for ch in component.chars() {
            sim.type_char(ch);
        }

        let after_first = sim.node_count();
        println!("After first component: {} nodes", after_first);

        // Type second component
        let second = r#"/**
 * @frame(x: 500, y: 0, width: 400, height: 300)
 */
component Second {
    render div { text "second" }
}"#;

        for ch in second.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 2, "Should have 2 components");
    }

    #[test]
    fn test_type_component_and_render() {
        let mut sim = TypingSimulator::new();

        // This is the user's exact scenario
        let content = r#"/**
 * @frame(x: 0, y: 0, width: 555, height: 505)
 */
component Card {
    render div {
        style {
            padding: 32px
            color: orange
        }
        text "hello world"
    }
}

/**
 * @frame(x: 600, y: 0, width: 800, height: 600)
 */
div {
    text "standalone render"
}"#;

        for ch in content.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 2, "Should have 2 nodes (component + render)");
    }

    // =========================================================================
    // SECTION 3: Editing Existing Content
    // =========================================================================

    #[test]
    fn test_edit_text_content() {
        let initial = r#"div { text "hello" }"#;
        let mut sim = TypingSimulator::with_content(initial);

        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 1);

        // Find position of "hello" and modify it
        // hello is at position 12 (after 'div { text "')
        let hello_start = 12;
        let hello_end = 17;

        // Delete "hello"
        sim.move_cursor(hello_end as u32);
        sim.backspace(5);

        // At this point: div { text "" } - might or might not parse

        // Type "world"
        for ch in "world".chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.content().contains("world"));
    }

    #[test]
    fn test_edit_preserves_frame() {
        let initial = r#"/**
 * @frame(x: 100, y: 200)
 */
div { text "original" }"#;

        let mut sim = TypingSimulator::with_content(initial);

        assert!(sim.has_frame(), "Initial state should have frame");
        let initial_frame = sim.get_frame().unwrap();

        // Edit the text (replace "original" with "modified")
        let content = sim.content();
        let start = content.find("original").unwrap();
        let end = start + "original".len();

        sim.replace(start as u32, end as u32, "modified");

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.has_frame(), "Frame should be preserved after edit");

        let final_frame = sim.get_frame().unwrap();
        assert_eq!(initial_frame, final_frame, "Frame values should be unchanged");
    }

    // =========================================================================
    // SECTION 4: Error Recovery Scenarios
    // =========================================================================

    #[test]
    fn test_syntax_error_preserves_state() {
        let valid = r#"div { text "hello" }"#;
        let mut sim = TypingSimulator::with_content(valid);

        assert!(sim.last_result().unwrap().parse_ok);
        let valid_node_count = sim.node_count();

        // Break the syntax by deleting closing quote
        // `div { text "hello" }` has quote at position 17
        // To delete it with backspace, cursor must be AFTER the quote (position 18)
        sim.move_cursor(18);
        sim.backspace(1);  // Delete the closing "

        // Parse should fail - we now have `div { text "hello }`
        assert!(!sim.last_result().unwrap().parse_ok,
            "Broken syntax should fail, content: {}", sim.content());

        // Node count should remain from valid state
        assert_eq!(sim.last_result().unwrap().node_count, valid_node_count,
            "Node count should be preserved from last valid state");
    }

    #[test]
    fn test_frame_persists_through_errors() {
        let with_frame = r#"/**
 * @frame(x: 50, y: 100, width: 200, height: 150)
 */
div { text "content" }"#;

        let mut sim = TypingSimulator::with_content(with_frame);

        assert!(sim.has_frame(), "Initial should have frame");
        let initial_frame = sim.get_frame().unwrap();

        // Break the syntax
        let content = sim.content();
        let quote_pos = content.rfind('"').unwrap() as u32;
        sim.move_cursor(quote_pos + 1);
        sim.backspace(1);  // Delete closing "

        assert!(!sim.last_result().unwrap().parse_ok);

        // Frame should still be there from last valid state
        assert!(sim.last_result().unwrap().has_frame,
            "Frame should persist during syntax error");

        // Fix the syntax
        sim.type_char('"');

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.has_frame(), "Frame should exist after recovery");
        assert_eq!(sim.get_frame().unwrap(), initial_frame, "Frame should be unchanged");
    }

    #[test]
    fn test_recovery_after_multiple_errors() {
        let valid = r#"component Test { render div { text "ok" } }"#;
        let mut sim = TypingSimulator::with_content(valid);

        assert!(sim.last_result().unwrap().parse_ok);
        let valid_nodes = sim.node_count();

        // Create a series of errors
        sim.move_to_end();

        // Type partial content that creates errors
        sim.type_str("\n\ncomponent"); // Incomplete
        assert!(!sim.last_result().unwrap().parse_ok);

        sim.type_str(" Broken"); // Still incomplete
        assert!(!sim.last_result().unwrap().parse_ok);

        sim.type_str(" {"); // Still incomplete
        assert!(!sim.last_result().unwrap().parse_ok);

        // During errors, node count should remain from valid state
        assert_eq!(sim.last_result().unwrap().node_count, valid_nodes);

        // Now complete it
        sim.type_str("\n    render div { text \"fixed\" }\n}");

        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 2, "Should now have 2 components");
    }

    // =========================================================================
    // SECTION 5: Stress Tests - Rapid Typing
    // =========================================================================

    #[test]
    fn test_rapid_typing_large_component() {
        let mut sim = TypingSimulator::new();

        let large_component = r#"/**
 * @frame(x: 0, y: 0, width: 800, height: 600)
 */
component Dashboard {
    render div {
        style {
            display: flex
            flex-direction: column
            padding: 24px
            background: #f5f5f5
        }
        div {
            style {
                font-size: 24px
                font-weight: bold
                margin-bottom: 16px
            }
            text "Dashboard Header"
        }
        div {
            style {
                display: flex
                gap: 16px
            }
            div {
                style { padding: 16px; background: white; border-radius: 8px; }
                text "Card 1"
            }
            div {
                style { padding: 16px; background: white; border-radius: 8px; }
                text "Card 2"
            }
            div {
                style { padding: 16px; background: white; border-radius: 8px; }
                text "Card 3"
            }
        }
    }
}"#;

        // Type it all character by character
        for ch in large_component.chars() {
            sim.type_char(ch);
        }

        println!("Typed {} chars: {} valid, {} errors",
            large_component.len(), sim.success_count(), sim.error_count());

        // Debug: show final content and error
        let result = sim.last_result().unwrap();
        if !result.parse_ok {
            println!("Final content ({} chars):\n{}", sim.content().len(), sim.content());
            println!("Error: {:?}", result.error);
        }

        // Final should be valid with frame
        assert!(sim.last_result().unwrap().parse_ok,
            "Final content should be valid, got error: {:?}", sim.last_result().unwrap().error);
        assert!(sim.has_frame());
        assert_eq!(sim.node_count(), 1);
    }

    #[test]
    fn test_rapid_backspace_and_retype() {
        let initial = r#"component Test {
    render div {
        text "hello world"
    }
}"#;

        let mut sim = TypingSimulator::with_content(initial);
        assert!(sim.last_result().unwrap().parse_ok);

        // Backspace the entire file
        let len = sim.content().len();
        for _ in 0..len {
            sim.backspace(1);
        }

        assert!(sim.content().is_empty());

        // Retype it
        for ch in initial.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok);
        assert_eq!(sim.node_count(), 1);
    }

    // =========================================================================
    // SECTION 6: Specific Bug Reproductions
    // =========================================================================

    #[test]
    fn test_user_scenario_component_and_div_frames() {
        // Exact reproduction of user's issue
        let mut sim = TypingSimulator::new();

        let content = r#"/**
 * @frame(x: 0, y: 0, width: 555, height: 505)
 */
component Card {
    render div {
        style {
            padding: 32px
            color: orange
            font-size: 32px
            font-weight: bold
            text-decoration: underline
        }
        text "hello worlddfdfsdd " {
          style {
            color: red
          }
        }
    }
}

/**
 * @frame(x: 0, y: 0, width: 1344, height: 1209)
 */
div {
    text "blaaahahhahaahhah"
}"#;

        // Type the whole thing
        for ch in content.chars() {
            sim.type_char(ch);
        }

        let result = sim.last_result().unwrap();

        assert!(result.parse_ok, "Final content should parse");
        assert_eq!(sim.node_count(), 2,
            "Should have 2 nodes: 1 component + 1 render. Got: {}", sim.node_count());

        // Both should have frames
        let file_state = sim.state.get_file(&sim.file_path).unwrap();
        for (i, node) in file_state.vdom.nodes.iter().enumerate() {
            if let VNode::Element { attributes, .. } = node {
                assert!(attributes.contains_key("data-frame-x"),
                    "Node {} should have data-frame-x", i);
                assert!(attributes.contains_key("data-frame-y"),
                    "Node {} should have data-frame-y", i);
            } else {
                panic!("Node {} should be Element", i);
            }
        }
    }

    #[test]
    fn test_typing_unclosed_quote_then_fixing() {
        let mut sim = TypingSimulator::new();

        // Type: text "hello
        sim.type_str("text \"hello");

        // This might or might not parse (parser may be lenient)
        let after_unclosed = sim.last_result().unwrap().clone();
        println!("After unclosed quote: parse_ok={}", after_unclosed.parse_ok);

        // Close the quote
        sim.type_char('"');

        assert!(sim.last_result().unwrap().parse_ok,
            "Should parse after closing quote");
    }

    #[test]
    fn test_typing_unclosed_brace_then_fixing() {
        let mut sim = TypingSimulator::new();

        // Type: div {
        sim.type_str("div {");

        assert!(!sim.last_result().unwrap().parse_ok,
            "Unclosed brace should fail");

        // Close it
        sim.type_str(" }");

        assert!(sim.last_result().unwrap().parse_ok,
            "Should parse after closing brace");
    }

    #[test]
    fn test_deleting_frame_annotation() {
        let with_frame = r#"/**
 * @frame(x: 100, y: 200)
 */
div { text "content" }"#;

        let mut sim = TypingSimulator::with_content(with_frame);

        assert!(sim.has_frame(), "Should have frame initially");

        // Replace content without frame
        let without_frame = r#"div { text "content" }"#;
        sim.set_content(without_frame);

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(!sim.has_frame(), "Frame should be gone after removing annotation");
    }

    #[test]
    fn test_adding_frame_annotation() {
        let without_frame = r#"div { text "content" }"#;
        let mut sim = TypingSimulator::with_content(without_frame);

        assert!(!sim.has_frame(), "Should not have frame initially");

        // Add frame annotation at beginning
        sim.move_cursor(0);
        sim.type_str("/**\n * @frame(x: 50, y: 75)\n */\n");

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.has_frame(), "Should have frame after adding annotation");

        let frame = sim.get_frame().unwrap();
        assert_eq!(frame.0, 50.0);
        assert_eq!(frame.1, 75.0);
    }

    // =========================================================================
    // SECTION 7: Multi-File Scenarios (Future)
    // =========================================================================

    #[test]
    fn test_multiple_sequential_edits() {
        // Simulate user making many small edits
        let mut sim = TypingSimulator::with_content(r#"div { text "a" }"#);

        // Replace "a" with "b", "c", "d", etc.
        for ch in 'b'..='z' {
            let content = sim.content();
            let pos = content.find('"').unwrap() + 1;
            sim.move_cursor(pos as u32 + 1);
            sim.backspace(1);
            sim.type_char(ch);

            assert!(sim.last_result().unwrap().parse_ok,
                "Should parse after replacing with '{}'", ch);
        }

        // Final content should have 'z'
        assert!(sim.content().contains("\"z\""));
    }

    // =========================================================================
    // SECTION 8: Edge Cases
    // =========================================================================

    #[test]
    fn test_empty_file() {
        let mut sim = TypingSimulator::new();

        // Empty might be valid (empty document)
        let result = sim.process();
        println!("Empty file: parse_ok={}", result.parse_ok);

        // Start typing
        sim.type_str("div {}");
        assert!(sim.last_result().unwrap().parse_ok);
    }

    #[test]
    fn test_whitespace_only() {
        let mut sim = TypingSimulator::with_content("   \n\n   \t  \n");

        // Whitespace only should be valid (empty document)
        let result = sim.last_result().unwrap();
        println!("Whitespace only: parse_ok={}", result.parse_ok);
    }

    #[test]
    fn test_unicode_content() {
        let mut sim = TypingSimulator::new();

        sim.type_str(r#"div { text "Hello 世界 🌍" }"#);

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.content().contains("世界"));
        assert!(sim.content().contains("🌍"));
    }

    #[test]
    fn test_very_long_line() {
        let mut sim = TypingSimulator::new();

        let long_text = "x".repeat(10000);
        sim.type_str(&format!(r#"div {{ text "{}" }}"#, long_text));

        assert!(sim.last_result().unwrap().parse_ok);
        assert!(sim.content().len() > 10000);
    }

    #[test]
    fn test_deeply_nested() {
        let mut sim = TypingSimulator::new();

        // Create deeply nested structure
        let mut content = String::new();
        for i in 0..20 {
            content.push_str(&format!("div {{ /* level {} */ ", i));
        }
        content.push_str(r#"text "deep""#);
        for _ in 0..20 {
            content.push_str(" }");
        }

        for ch in content.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok,
            "Deeply nested should parse");
    }

    // =========================================================================
    // SECTION 9: Patch Verification
    // =========================================================================

    #[test]
    fn test_initial_load_generates_initialize_patch() {
        let content = r#"div { text "hello" }"#;
        let mut sim = TypingSimulator::new();

        // Type content
        for ch in content.chars() {
            sim.type_char(ch);
        }

        // Look through history for initialize patch
        let has_initialize = sim.patch_history.iter()
            .any(|r| r.parse_ok && r.patch_count > 0);

        assert!(has_initialize, "Should have generated patches");
    }

    #[test]
    fn test_incremental_edits_generate_patches() {
        let mut sim = TypingSimulator::with_content(r#"div { text "a" }"#);

        // Make incremental edit
        let content = sim.content();
        let pos = content.find('a').unwrap();
        sim.move_cursor(pos as u32 + 1);
        sim.backspace(1);
        sim.type_char('b');

        // Should have generated patch
        let result = sim.last_result().unwrap();
        assert!(result.parse_ok);
        // Note: patch_count might be 0 if the text change didn't affect VDOM structure
        println!("Incremental edit generated {} patches", result.patch_count);
    }

    // =========================================================================
    // REGRESSION: Adding style block to text element causes frame loss
    // =========================================================================

    #[test]
    fn test_adding_style_to_text_preserves_frame() {
        // Bug: Adding style block to text element causes frame to disappear
        // Scenario: User types `text "hello" { }` then adds style block inside

        // Step 1: Initial state with text and empty braces
        let initial = r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
    }
    text "hello" {
    }
}"#;
        let mut sim = TypingSimulator::with_content(initial);

        assert!(sim.last_result().unwrap().parse_ok, "Initial should parse");
        assert!(sim.has_frame(), "Initial should have frame");
        let initial_frame = sim.get_frame();
        println!("Initial frame: {:?}", initial_frame);

        // Step 2: Add style block inside text braces
        // Find position just before the closing brace of text element
        let content = sim.content();
        // Find "text \"hello\" {" and position after the opening brace
        let text_brace_pos = content.find("text \"hello\" {").unwrap() + "text \"hello\" {".len();
        sim.move_cursor(text_brace_pos as u32);

        // Type the style block
        sim.type_str("\n        style {\n            color: red\n        }\n    ");

        println!("After adding style: parse_ok={}, has_frame={}",
            sim.last_result().unwrap().parse_ok,
            sim.has_frame());
        println!("Content:\n{}", sim.content());

        // Should still parse and have frame
        assert!(sim.last_result().unwrap().parse_ok,
            "Should parse after adding style block");
        assert!(sim.has_frame(),
            "Frame should persist after adding style to text element");
        assert_eq!(sim.get_frame(), initial_frame,
            "Frame coordinates should be unchanged");
    }

    #[test]
    fn test_text_with_style_block_from_scratch() {
        // Test building text with style block from scratch
        let mut sim = TypingSimulator::new();

        let content = r#"/**
 * @frame(x: 50, y: 50, width: 300, height: 200)
 */
div {
    text "styled text" {
        style {
            color: blue
            font-size: 24px
        }
    }
}"#;

        // Type character by character
        for ch in content.chars() {
            sim.type_char(ch);
        }

        assert!(sim.last_result().unwrap().parse_ok,
            "Text with style block should parse");
        assert!(sim.has_frame(),
            "Should have frame after typing complete");

        let frame = sim.get_frame();
        assert!(frame.is_some(), "Frame should exist");
        let (x, y, _, _) = frame.unwrap();
        assert_eq!(x, 50.0, "Frame x should be 50");
        assert_eq!(y, 50.0, "Frame y should be 50");
    }

    #[test]
    fn test_multiple_text_elements_with_styles() {
        // Multiple text elements, each with style blocks
        let content = r#"/**
 * @frame(x: 0, y: 0, width: 600, height: 400)
 */
div {
    text "first" {
        style { color: red }
    }
    text "second" {
        style { color: blue }
    }
    text "third" {
        style { color: green }
    }
}"#;

        let mut sim = TypingSimulator::with_content(content);

        assert!(sim.last_result().unwrap().parse_ok,
            "Multiple styled text elements should parse");
        assert!(sim.has_frame(),
            "Should have frame");
    }

    #[test]
    fn test_user_bug_report_text_style_frame_loss() {
        // Exact user scenario:
        // Before: text "hello wodrlddddd" { }
        // After: text "hello wodrlddddd" { style { color: red } }
        // Bug: frame disappears and never comes back

        // Initial state - note the user likely has a frame annotation
        let initial = r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
        color: orange
        font-size: 32px
        font-weight: bold
        text-decoration: underline
    }
    text "hello wodrlddddd" {
    }
}"#;

        let mut sim = TypingSimulator::with_content(initial);
        println!("Step 1 - Initial:");
        println!("  parse_ok: {}", sim.last_result().unwrap().parse_ok);
        println!("  has_frame: {}", sim.has_frame());
        println!("  node_count: {}", sim.node_count());
        assert!(sim.has_frame(), "Initial should have frame");

        // Final state after adding style
        let final_content = r#"/**
 * @frame(x: 100, y: 100, width: 400, height: 300)
 */
div {
    style {
        padding: 32px
        color: orange
        font-size: 32px
        font-weight: bold
        text-decoration: underline
    }
    text "hello wodrlddddd" {
        style {
            color: red
        }
    }
}"#;

        sim.set_content(final_content);
        println!("\nStep 2 - After adding style:");
        println!("  parse_ok: {}", sim.last_result().unwrap().parse_ok);
        println!("  has_frame: {}", sim.has_frame());
        println!("  node_count: {}", sim.node_count());

        assert!(sim.last_result().unwrap().parse_ok, "Final should parse");
        assert!(sim.has_frame(), "Frame should persist after adding style to text");
    }
}
