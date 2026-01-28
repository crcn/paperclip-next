# Evaluator Logging

The evaluator uses the `tracing` crate for structured logging. This provides detailed insights into the evaluation process and helps with debugging.

## Log Levels

The evaluator emits logs at different levels:

- **ERROR**: Critical errors (component not found, evaluation failures)
- **WARN**: Warnings (variable not found, type mismatches)
- **INFO**: High-level operations (document evaluation start/end, component registration)
- **DEBUG**: Detailed operations (token registration, component evaluation, patch generation)

## Setting Up Logging

To enable logging in your application, initialize a tracing subscriber:

```rust
use tracing_subscriber;

// Initialize with default settings (shows INFO and above)
tracing_subscriber::fmt::init();

// Or with custom settings
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .with_target(false)
    .init();
```

## Example Output

```
INFO evaluate{components=2 tokens=3}: paperclip_evaluator::evaluator: Starting document evaluation
DEBUG evaluate{components=2 tokens=3}: paperclip_evaluator::evaluator: token_name="primary-color" token_value="#FF0000" Registering token
DEBUG evaluate{components=2 tokens=3}: paperclip_evaluator::evaluator: component_name="Button" public=true Registering component
INFO evaluate{components=2 tokens=3}: paperclip_evaluator::evaluator: public_components=1 Evaluating public components
DEBUG evaluate{components=2 tokens=3}: paperclip_evaluator::evaluator: component_name="Button" Evaluating public component
INFO evaluate{components=2 tokens=3}: paperclip_evaluator::evaluator: nodes=1 Document evaluation complete
```

## Error Context

All errors include span information showing where in the source the error occurred:

```rust
EvalError::VariableNotFound {
    name: "undefinedVar".to_string(),
    span: Span::new(45, 58),
}
```

Error messages are descriptive and include:
- The variable/component name that caused the error
- The span (position) in the source code
- Additional context (e.g., expected vs actual types)

## Performance Logging

Use the `instrument` attribute to trace function execution:

```rust
#[instrument(skip(self), fields(component_name = name))]
fn evaluate_component(&self, name: &str) -> EvalResult<VNode> {
    // Automatically logs entry and exit with timing
}
```

## Integration with Workspace

The workspace server also uses tracing for logging:

```rust
// In state.rs
info!(is_cached, "Updating file");
debug!("Parsing source");
error!(error = ?e, "Parse failed");
```

This provides end-to-end visibility from file changes through evaluation to patch generation.
