//! CSS minification - compress CSS for smaller payloads

use crate::vdom::CssRule;

/// Minify CSS property values
pub fn minify_css_value(value: &str) -> String {
    let trimmed = value.trim();

    // Remove unnecessary whitespace
    let mut result = String::with_capacity(trimmed.len());
    let mut last_was_space = false;

    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }

    // Minify common patterns
    // Replace zero values (with word boundaries)
    result = result
        .replace(" 0px", " 0")
        .replace(" 0em", " 0")
        .replace(" 0rem", " 0");

    // Handle zero at start of string
    if result.starts_with("0px") {
        result = format!("0{}", &result[3..]);
    } else if result.starts_with("0em") {
        result = format!("0{}", &result[3..]);
    } else if result.starts_with("0rem") {
        result = format!("0{}", &result[4..]);
    }

    result = result
        // Colors: #ffffff -> #fff
        .replace("#ffffff", "#fff")
        .replace("#000000", "#000")
        // Remove spaces around operators
        .replace(" + ", "+")
        .replace(" - ", "-")
        .replace(" * ", "*")
        .replace(" / ", "/")
        // Trim
        .trim()
        .to_string();

    result
}

/// Minify a CSS selector
pub fn minify_css_selector(selector: &str) -> String {
    selector.trim()
        // Remove spaces around combinators
        .replace(" > ", ">")
        .replace(" + ", "+")
        .replace(" ~ ", "~")
        .to_string()
}

/// Minify CSS rules in-place
pub fn minify_css_rules(rules: &mut [CssRule]) {
    for rule in rules.iter_mut() {
        // Minify selector
        rule.selector = minify_css_selector(&rule.selector);

        // Minify all property values
        for value in rule.properties.values_mut() {
            *value = minify_css_value(value);
        }

        // Minify media query if present
        if let Some(ref mut mq) = rule.media_query {
            *mq = mq.trim().to_string();
        }
    }
}

/// Calculate compression ratio
pub fn calculate_compression_ratio(original_size: usize, minified_size: usize) -> f64 {
    if original_size == 0 {
        return 0.0;
    }
    ((original_size - minified_size) as f64 / original_size as f64) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_minify_zero_values() {
        assert_eq!(minify_css_value("0px"), "0");
        assert_eq!(minify_css_value("0em"), "0");
        assert_eq!(minify_css_value("0rem"), "0");
        assert_eq!(minify_css_value("10px"), "10px"); // Don't touch non-zero
    }

    #[test]
    fn test_minify_colors() {
        assert_eq!(minify_css_value("#ffffff"), "#fff");
        assert_eq!(minify_css_value("#000000"), "#000");
        assert_eq!(minify_css_value("#123456"), "#123456"); // Don't touch non-minifiable
    }

    #[test]
    fn test_remove_whitespace() {
        assert_eq!(minify_css_value("  10px  20px  "), "10px 20px");
        assert_eq!(minify_css_value("calc(100% - 20px)"), "calc(100%-20px)");
    }

    #[test]
    fn test_minify_selector() {
        assert_eq!(minify_css_selector(".foo > .bar"), ".foo>.bar");
        assert_eq!(minify_css_selector(".a + .b"), ".a+.b");
    }

    #[test]
    fn test_minify_rules() {
        let mut rules = vec![
            CssRule {
                selector: ".foo > .bar".to_string(),
                properties: [
                    ("padding".to_string(), "0px".to_string()),
                    ("margin".to_string(), "10px  20px".to_string()),
                    ("color".to_string(), "#ffffff".to_string()),
                ].into(),
                media_query: None,
            }
        ];

        minify_css_rules(&mut rules);

        assert_eq!(rules[0].selector, ".foo>.bar");
        assert_eq!(rules[0].properties.get("padding"), Some(&"0".to_string()));
        assert_eq!(rules[0].properties.get("margin"), Some(&"10px 20px".to_string()));
        assert_eq!(rules[0].properties.get("color"), Some(&"#fff".to_string()));
    }
}
