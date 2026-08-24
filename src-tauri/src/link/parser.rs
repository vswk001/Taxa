// src-tauri/src/link/parser.rs
use regex::Regex;
use std::sync::OnceLock;

fn link_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[\[([^\]]+)\]\]").expect("invalid link regex"))
}

pub struct LinkParser;

impl LinkParser {
    /// Extracts all [[link]] patterns from markdown content.
    /// Handles both raw `[[link]]` and remark-escaped `\[\[link\]\]`.
    /// Duplicates are removed, preserving first-seen order.
    pub fn extract_links(content: &str) -> Vec<String> {
        // Unescape markdown backslash escapes for brackets before matching
        let unescaped = content.replace("\\[", "[").replace("\\]", "]");
        let mut seen = std::collections::HashSet::new();
        link_regex()
            .captures_iter(&unescaped)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .filter(|link| seen.insert(link.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_and_dedups() {
        let links = LinkParser::extract_links("See [[A]] and \\[\\[B\\]\\] then [[A]] again");
        assert_eq!(links, vec!["A".to_string(), "B".to_string()]);
    }
}
