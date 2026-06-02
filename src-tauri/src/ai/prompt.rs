// src-tauri/src/ai/prompt.rs
use crate::ai::provider::Message;

pub struct PromptTemplates;

impl PromptTemplates {
    pub fn categorize(content: &str, folder_structure: &str, related_notes: &str) -> Vec<Message> {
        vec![
            Message {
                role: "system".into(),
                content: format!(
                    "你是一个笔记归类助手。用户会给你一段内容，你需要决定如何处理它。\n\
                    当前笔记库的目录结构：\n{}\n\n\
                    相关已有笔记：\n{}\n\n\
                    你需要返回JSON格式（不要markdown代码块）：\n\
                    {{\"action\": \"create\" | \"append\", \"title\": \"笔记标题\", \
                    \"folder\": \"归类到的文件夹路径\", \"tags\": [\"标签1\", \"标签2\"], \
                    \"content\": \"完善后的内容\", \"target_note_id\": \"如果append则填目标笔记ID\", \
                    \"complexity\": \"simple\" | \"complex\"}}\n\
                    - action: create=新建笔记, append=追加到已有笔记\n\
                    - complexity: simple=简短内容可直接应用, complex=复杂内容需用户确认\n\
                    - folder: 必须是目录结构中已有的或合理的新路径\n\
                    - content: 对用户内容进行适当的完善、格式化、补充\n\
                    - tags: 建议的标签",
                    folder_structure, related_notes
                ),
            },
            Message {
                role: "user".into(),
                content: content.to_string(),
            },
        ]
    }

    pub fn enrich(title: &str, content: &str) -> Vec<Message> {
        vec![
            Message {
                role: "system".into(),
                content: "你是一个笔记内容助手。用户会给你一篇笔记的标题和内容，你需要：\n\
                1. 完善内容（补充缺失信息、改善格式、修正错别字）\n\
                2. 生成一个简洁的摘要（不超过100字）\n\
                3. 建议合适的标签\n\n\
                返回JSON格式（不要markdown代码块）：\n\
                {\"title\": \"建议的标题\", \"content\": \"完善后的内容\", \
                \"summary\": \"摘要\", \"tags\": [\"标签\"]}".into(),
            },
            Message {
                role: "user".into(),
                content: format!("标题：{}\n\n内容：\n{}", title, content),
            },
        ]
    }

    pub fn rename(title: &str, content: &str) -> Vec<Message> {
        vec![
            Message {
                role: "system".into(),
                content: "根据笔记内容，建议一个简洁准确的标题。只返回标题文本，不要其他内容。".into(),
            },
            Message {
                role: "user".into(),
                content: format!("当前标题：{}\n\n内容摘要：\n{}", title, &content[..content.len().min(500)]),
            },
        ]
    }
}
