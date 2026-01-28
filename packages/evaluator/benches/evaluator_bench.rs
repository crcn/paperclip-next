use criterion::{black_box, criterion_group, criterion_main, Criterion};
use paperclip_evaluator::Evaluator;
use paperclip_parser::parse;

fn evaluate_simple_component(c: &mut Criterion) {
    let source = r#"
        public component Button {
            render button {
                style {
                    padding: 8px 16px
                    background: #3366FF
                }
                text "Click me"
            }
        }
    "#;

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_simple_component", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_medium_component(c: &mut Criterion) {
    let source = r#"
        public token primaryColor #3366FF
        public token spacing 16px

        public component Card {
            render div {
                style {
                    padding: 16px
                    background: white
                    border-radius: 8px
                }

                div {
                    style {
                        font-size: 24px
                        font-weight: bold
                        margin-bottom: 8px
                    }
                    text "Card Title"
                }

                div {
                    style {
                        color: #666
                        line-height: 1.5
                    }
                    text "Card description"
                }

                button {
                    style {
                        padding: 8px 16px
                        background: #3366FF
                        color: white
                        border: none
                    }
                    text "Action"
                }
            }
        }
    "#;

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_medium_component", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_multiple_components(c: &mut Criterion) {
    let mut source = String::new();

    // Add multiple components
    for i in 0..10 {
        source.push_str(&format!(
            r#"
            public component Component{} {{
                render div {{
                    style {{
                        padding: 16px
                    }}
                    div {{
                        text "Component {}"
                    }}
                    button {{
                        text "Click"
                    }}
                }}
            }}
            "#,
            i, i
        ));
    }

    let doc = parse(&source).unwrap();

    c.bench_function("evaluate_10_components", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn parse_and_evaluate(c: &mut Criterion) {
    let source = r#"
        public component Card {
            render div {
                style {
                    padding: 16px
                    background: white
                }
                div {
                    text "Hello"
                }
                button {
                    text "Click"
                }
            }
        }
    "#;

    c.bench_function("parse_and_evaluate_combined", |b| {
        b.iter(|| {
            let doc = parse(black_box(source)).unwrap();
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(&doc)
        })
    });
}

fn evaluate_component_with_props(c: &mut Criterion) {
    let source = r#"
        component Greeting {
            render div {
                style {
                    padding: 16px
                    font-size: 18px
                }
                text {name}
            }
        }

        public component App {
            render div {
                Greeting(name="Alice")
                Greeting(name="Bob")
                Greeting(name="Charlie")
            }
        }
    "#;

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_component_expansion_with_props", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_deeply_nested(c: &mut Criterion) {
    let source = r#"
        public component DeepNesting {
            render div {
                div {
                    div {
                        div {
                            div {
                                div {
                                    div {
                                        div {
                                            div {
                                                div {
                                                    text "Deep content"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    "#;

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_deeply_nested_10_levels", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_many_siblings(c: &mut Criterion) {
    let mut source = String::from(
        r#"
        public component ManySiblings {
            render div {
        "#,
    );

    // Add 50 sibling elements
    for i in 0..50 {
        source.push_str(&format!(
            r#"
                div {{
                    text "Item {}"
                }}
            "#,
            i
        ));
    }

    source.push_str(
        r#"
            }
        }
        "#,
    );

    let doc = parse(&source).unwrap();

    c.bench_function("evaluate_50_sibling_elements", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn evaluate_with_many_styles(c: &mut Criterion) {
    let source = r#"
        public component StyledComponent {
            render div {
                style {
                    padding: 16px
                    margin: 8px
                    background: #FF0000
                    color: white
                    font-size: 14px
                    font-weight: bold
                    border-radius: 4px
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1)
                    display: flex
                    align-items: center
                }
                text "Styled content"
            }
        }
    "#;

    let doc = parse(source).unwrap();

    c.bench_function("evaluate_component_with_many_styles", |b| {
        b.iter(|| {
            let mut evaluator = Evaluator::new();
            evaluator.evaluate(black_box(&doc))
        })
    });
}

fn diff_large_vdocument(c: &mut Criterion) {
    use paperclip_evaluator::diff_vdocument;

    // Create two large documents with slight differences
    let source1 = (0..20)
        .map(|i| {
            format!(
                r#"
                public component Comp{} {{
                    render div {{
                        text "Component {}"
                        button {{
                            text "Button {}"
                        }}
                    }}
                }}
            "#,
                i, i, i
            )
        })
        .collect::<String>();

    let source2 = (0..20)
        .map(|i| {
            format!(
                r#"
                public component Comp{} {{
                    render div {{
                        text "Component {} Updated"
                        button {{
                            text "Button {}"
                        }}
                    }}
                }}
            "#,
                i, i, i
            )
        })
        .collect::<String>();

    let doc1 = parse(&source1).unwrap();
    let doc2 = parse(&source2).unwrap();

    let mut evaluator1 = Evaluator::new();
    let mut evaluator2 = Evaluator::new();

    let vdoc1 = evaluator1.evaluate(&doc1).unwrap();
    let vdoc2 = evaluator2.evaluate(&doc2).unwrap();

    c.bench_function("diff_20_components_with_changes", |b| {
        b.iter(|| diff_vdocument(black_box(&vdoc1), black_box(&vdoc2)))
    });
}

fn diff_identical_vdocuments(c: &mut Criterion) {
    use paperclip_evaluator::diff_vdocument;

    let source = (0..20)
        .map(|i| {
            format!(
                r#"
                public component Comp{} {{
                    render div {{
                        text "Component {}"
                    }}
                }}
            "#,
                i, i
            )
        })
        .collect::<String>();

    let doc = parse(&source).unwrap();

    let mut evaluator = Evaluator::new();
    let vdoc = evaluator.evaluate(&doc).unwrap();

    c.bench_function("diff_identical_20_components", |b| {
        b.iter(|| {
            // Should be fast since nothing changed
            diff_vdocument(black_box(&vdoc), black_box(&vdoc))
        })
    });
}

criterion_group!(
    benches,
    evaluate_simple_component,
    evaluate_medium_component,
    evaluate_multiple_components,
    parse_and_evaluate,
    evaluate_component_with_props,
    evaluate_deeply_nested,
    evaluate_many_siblings,
    evaluate_with_many_styles,
    diff_large_vdocument,
    diff_identical_vdocuments
);
criterion_main!(benches);
