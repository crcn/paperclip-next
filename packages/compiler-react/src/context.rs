use std::cell::RefCell;
use std::rc::Rc;

/// Options for React compilation
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Whether to use TypeScript types
    pub use_typescript: bool,
    /// Whether to include CSS imports
    pub include_css_imports: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            use_typescript: false,
            include_css_imports: true,
        }
    }
}

/// Compilation context for managing state during code generation
pub struct CompilerContext {
    buffer: Rc<RefCell<String>>,
    indent_level: Rc<RefCell<usize>>,
    pub options: CompileOptions,
}

impl CompilerContext {
    pub fn new(options: CompileOptions) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(String::new())),
            indent_level: Rc::new(RefCell::new(0)),
            options,
        }
    }

    pub fn add(&self, text: &str) {
        self.buffer.borrow_mut().push_str(text);
    }

    pub fn add_line(&self, text: &str) {
        self.add_indented(text);
        self.add("\n");
    }

    pub fn add_indented(&self, text: &str) {
        let indent = "  ".repeat(*self.indent_level.borrow());
        self.buffer.borrow_mut().push_str(&indent);
        self.buffer.borrow_mut().push_str(text);
    }

    pub fn indent(&self) {
        *self.indent_level.borrow_mut() += 1;
    }

    pub fn dedent(&self) {
        let mut level = self.indent_level.borrow_mut();
        if *level > 0 {
            *level -= 1;
        }
    }

    pub fn get_output(&self) -> String {
        self.buffer.borrow().clone()
    }

    pub fn with_new_buffer(&self) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(String::new())),
            indent_level: self.indent_level.clone(),
            options: self.options.clone(),
        }
    }

    pub fn merge_buffer(&self, other: &CompilerContext) {
        self.buffer.borrow_mut().push_str(&other.buffer.borrow());
    }
}
