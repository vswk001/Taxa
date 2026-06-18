// src-tauri/src/bin/mcp/snippet.rs
// Extract a short content window around the first case-insensitive match of
// the query. Char-based so it is safe for CJK. Lowercase-expansion edge cases
// (e.g. ß, Turkish İ) are approximated, which is fine for previews.

/// Returns `None` if the query is empty or not found. Otherwise a `…window…`
/// slice of roughly `window` chars on each side of the match.
pub fn extract_window(content: &str, query: &str, window: usize) -> Option<String> {
    if query.trim().is_empty() {
        return None;
    }
    let lower = content.to_lowercase();
    let ql = query.to_lowercase();
    let mbyte = lower.find(&ql)?;

    // Map the byte offset back to a char offset in the original content.
    let start_char_idx = content[..mbyte].chars().count();
    let match_char_len = query.chars().count();
    let total_chars = content.chars().count();

    let start = start_char_idx.saturating_sub(window);
    let end = (start_char_idx + match_char_len + window).min(total_chars);

    let body: String = content.chars().skip(start).take(end - start).collect();
    let mut out = String::new();
    if start > 0 {
        out.push('…');
    }
    out.push_str(&body);
    if end < total_chars {
        out.push('…');
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_ascii_substring_case_insensitive() {
        let s = "This is a Rust note about async runtime and tokio.";
        let snip = extract_window(s, "async", 6).unwrap();
        assert!(snip.contains("async"));
        assert!(snip.starts_with('…') || snip.starts_with(' '));
    }

    #[test]
    fn finds_cjk_substring() {
        let s = "这是一篇关于异步运行时的笔记，内容涉及 tokio。";
        let snip = extract_window(s, "异步", 4).unwrap();
        assert!(snip.contains("异步"));
    }

    #[test]
    fn returns_none_when_absent() {
        assert!(extract_window("hello world", "zzz", 5).is_none());
    }

    #[test]
    fn returns_none_for_empty_query() {
        assert!(extract_window("hello", "", 5).is_none());
    }
}
