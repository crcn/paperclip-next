# Evaluator Performance Benchmarks

Comprehensive performance benchmarks for the Paperclip evaluator, measuring evaluation speed across various scenarios.

## Benchmark Results

All benchmarks run on:
- Hardware: Apple Silicon
- Rust: 1.x (release mode with optimizations)
- Criterion.rs for statistical analysis

### Core Evaluation Performance

| Benchmark | Time (µs) | Description |
|-----------|-----------|-------------|
| Simple Component | 1.00 | Single button with style and text |
| Medium Component | 4.03 | Card with multiple nested elements |
| 10 Components | 46.13 | Evaluating 10 public components |
| Parse + Evaluate | 2.62 | Combined parse and evaluate |

### Component Expansion (New Feature)

| Benchmark | Time (µs) | Description |
|-----------|-----------|-------------|
| Component with Props | 5.84 | Component instances with prop binding |

This demonstrates that component expansion with props adds minimal overhead (~4.8µs for 3 instances).

### Structural Complexity

| Benchmark | Time (µs) | Description |
|-----------|-----------|-------------|
| Deeply Nested (10 levels) | 2.14 | Deep element nesting |
| 50 Sibling Elements | 15.28 | Wide tree structure |
| Many Styles | 2.55 | Component with 10+ style properties |

**Key Insight**: Depth is cheaper than breadth. Deep nesting (10 levels) is faster than 50 siblings, suggesting that horizontal traversal is the primary cost.

### VDocument Diffing Performance

| Benchmark | Time (µs) | Description |
|-----------|-----------|-------------|
| Diff with Changes | 8.21 | 20 components with text changes |
| Diff Identical | 2.16 | 20 identical components (no changes) |

**Key Insight**: Diffing identical documents is ~4x faster than detecting changes, showing the optimization works correctly.

## Performance Characteristics

### Scaling

- **Linear scaling**: Adding components scales linearly (~4.6µs per component)
- **Efficient diffing**: Unchanged content diffing is very fast (2.1µs for 20 components)
- **Prop overhead**: Component expansion with props adds ~2µs per instance

### Bottlenecks

1. **Width over depth**: 50 siblings (15.28µs) vs 10 nesting levels (2.14µs)
2. **Change detection**: Diffing with changes (8.21µs) vs identical (2.16µs)
3. **Multiple components**: 10 components (46.13µs) suggests component registration overhead

## Optimization Opportunities

### High Priority
- **Component lookup**: HashMap lookup for components could be optimized
- **Sibling iteration**: Wide trees are more expensive, consider batch processing

### Medium Priority
- **Style processing**: 10 styles only adds 1.5µs, already efficient
- **Diff algorithm**: Already optimized for unchanged content

### Low Priority
- **Deep nesting**: Already very fast (2.14µs for 10 levels)
- **Parse integration**: Combined parse+eval is efficient (2.62µs)

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p paperclip-evaluator

# Run specific benchmark
cargo bench -p paperclip-evaluator --bench evaluator_bench -- evaluate_simple_component

# Save baseline for comparison
cargo bench -p paperclip-evaluator -- --save-baseline main
```

## Interpreting Results

Criterion.rs provides:
- **Mean time**: Average execution time
- **Outliers**: Statistical outliers detected
- **Regression detection**: Compares against previous runs

Look for:
- ✅ **Consistent times**: Low variance indicates stable performance
- ⚠️ **Outliers**: High outlier count may indicate GC or OS interference
- 🚨 **Regressions**: Significant slowdowns from previous baselines

## Recommendations

For production use:
- **< 100 components**: Excellent performance (~0.5ms total)
- **100-1000 components**: Good performance (~5ms total)
- **> 1000 components**: Consider chunking or lazy evaluation

The evaluator is highly optimized for typical UI component trees with reasonable nesting and component counts.
