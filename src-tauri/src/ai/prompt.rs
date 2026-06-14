// src-tauri/src/ai/prompt.rs
use crate::ai::provider::Message;

pub struct PromptTemplates;

impl PromptTemplates {
    pub fn categorize(raw_content: &str, folder_structure: &str, related_notes: &str) -> Vec<Message> {
        // Sanitize: remove null bytes, limit content length
        let content: String = raw_content
            .replace('\0', "")
            .chars()
            .take(80000)
            .collect();
        vec![
            Message {
                role: "system".into(),
                content: format!(
                    "你是一个笔记归类助手。用户会给你一段内容，你需要决定如何处理它。\n\n\
                    ## 核心规则：优先追加，避免重复\n\
                    当「相关已有笔记」中存在与用户输入**同一主题**的笔记时，必须选择 action=append，将新内容整合到该笔记中。\n\
                    只有在确认没有任何相关笔记时，才选择 action=create。\n\
                    判断标准：如果用户多次输入同一类内容（如多次输入游泳技巧），这些内容应该合并到同一个笔记中。\n\n\
                    ## 当前笔记库目录结构\n\
                    {folder_structure}\n\n\
                    ## 相关已有笔记（包含笔记ID、标题和内容摘要）\n\
                    {related_notes}\n\n\
                    ## 返回格式\n\
                    返回纯JSON（不要markdown代码块）：\n\
                    {{\"action\": \"create\" | \"append\", \"title\": \"笔记标题\", \
                    \"folder\": \"归类到的文件夹路径\", \"tags\": [\"标签1\", \"标签2\"], \
                    \"content\": \"完善后的内容\", \"target_note_id\": \"如果append则填目标笔记ID\", \
                    \"complexity\": \"simple\" | \"complex\"}}\n\n\
                    ## 字段说明\n\
                    - action: append=追加到已有笔记（优先），create=新建笔记（仅在无相关笔记时）\n\
                    - title: append时填目标笔记的原标题，create时填新笔记标题\n\
                    - target_note_id: append时必须填写目标笔记的id字段\n\
                    - folder: 必须是目录结构中已有的或合理的新路径\n\
                    - content: append时只填新增部分（系统会自动追加到原有内容后）；create时填完整内容\n\
                    - complexity: simple=简短内容可直接应用, complex=复杂内容需用户确认\n\
                    - tags: 建议的标签（合并已有笔记的标签和新标签）",
                    folder_structure = folder_structure,
                    related_notes = related_notes,
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

    pub fn optimize(title: &str, content: &str, instruction: &str) -> Vec<Message> {
        let content_clean: String = content.replace('\0', "").chars().take(80000).collect();
        let instruction_clean = if instruction.trim().is_empty() {
            "请全面优化这篇笔记".to_string()
        } else {
            instruction.to_string()
        };
        vec![
            Message {
                role: "system".into(),
                content: "你是一个笔记优化助手。用户会给你一篇已有笔记的标题和内容，以及优化指令。\n\
                你需要根据指令对笔记内容进行修改优化，保持笔记的核心信息不变。\n\n\
                返回纯JSON（不要markdown代码块）：\n\
                {\"title\": \"优化后的标题\", \"content\": \"优化后的完整内容\", \
                \"summary\": \"变更说明（简述做了哪些修改）\"}\n\n\
                要求：\n\
                - title: 如果优化指令未涉及标题，保持原标题不变\n\
                - content: 返回完整的优化后内容（不是增量）\n\
                - summary: 简述你做了哪些修改（一句话）\n\
                - 保持原有的markdown格式\n\
                - 不要添加原文中没有的信息，除非指令明确要求".into(),
            },
            Message {
                role: "user".into(),
                content: format!(
                    "标题：{}\n\n优化指令：{}\n\n笔记内容：\n{}",
                    title, instruction_clean, content_clean
                ),
            },
        ]
    }
}
