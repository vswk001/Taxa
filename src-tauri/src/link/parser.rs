// src-tauri/src/link/parser.rs
use regex::Regex;

pub struct LinkParser;

impl LinkParser {
    /// Extracts all [[link]] patterns from markdown content
    pub fn extract_links(content: &str) -> Vec<String> {
        let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
        re.captures_iter(content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    /// Finds backlinks by searching for [[note_title]] across all notes
    pub fn find_backlinks(note_title: &str, all_notes: &[(String, String)]) -> Vec<String> {
        let target = format!("[[{}]]", note_title);
        all_notes.iter()
            .filter(|(_, content)| content.contains(&target))
            .map(|(id, _)| id.clone())
            .collect()
    }
}
