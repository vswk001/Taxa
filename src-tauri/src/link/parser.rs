// src-tauri/src/link/parser.rs
use regex::Regex;

pub struct LinkParser;

impl LinkParser {
    /// Extracts all [[link]] patterns from markdown content
    /// Handles both raw `[[link]]` and remark-escaped `\[\[link\]\]`
    pub fn extract_links(content: &str) -> Vec<String> {
        // Unescape markdown backslash escapes for brackets before matching
        let unescaped = content.replace("\\[", "[").replace("\\]", "]");
        let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        re.captures_iter(&unescaped)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }
}
