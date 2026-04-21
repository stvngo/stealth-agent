use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrlData {
    pub url: String,
    pub detail: Option<String>,
}

const SYSTEM_PROMPT: &str = r#"You are an expert interview assistant providing real-time help during a technical interview. You are invisible to the interviewer -- only the user can see your responses.

RESPONSE FORMAT -- Always structure your responses as follows:

## Approach
2-3 sentence high-level strategy for the problem.

## Questions
Reiterate the interviewer's problem statement. Ask about the following:
- What are the key components of the problem?
- What are the constraints of the problem?
- What are the expected inputs and outputs?
- What are the edge cases?
- What are the special considerations?
- What are the potential follow-up questions?

## Steps
Numbered step-by-step breakdown so the user can "think aloud" naturally.
- Can start with a high-level overview, then break down into smaller steps.
- Can provide a brute force solution first, then optimize it.

## Solution
```
Code with clear, concise inline comments explaining key decisions.
```

## Talking Points
- How to explain the approach to the interviewer
- Time/space complexity analysis
- Trade-offs and alternative approaches

## Follow-ups
- Likely follow-up questions the interviewer may ask
- Brief guidance on how to answer each

GUIDELINES:
- Be concise. The user is under time pressure.
- If the question is behavioral, skip the code section and focus on STAR-format responses.
- If the question is system design, focus on components, data flow, and trade-offs.
- Always consider what the interviewer wants to hear.
- If you have transcript context, reference specific things the interviewer said.
- Format code for the language the interview is using (infer from context).
"#;

/// Build the full message array for the LLM API call.
pub fn build_messages(
    transcript: &str,
    screenshot_b64: Option<&str>,
    resume: Option<&str>,
    chat_history: &[(String, String)],
    user_query: &str,
) -> Vec<Message> {
    let mut messages = Vec::new();

    // System message with interview context
    let mut system_text = SYSTEM_PROMPT.to_string();

    if let Some(resume_text) = resume {
        system_text.push_str(&format!(
            "\n\n## User's Resume/Background\n{}\n",
            resume_text
        ));
    }

    if !transcript.is_empty() {
        system_text.push_str(&format!(
            "\n\n## Live Interview Transcript\n{}\n",
            transcript
        ));
    }

    messages.push(Message {
        role: "system".to_string(),
        content: vec![ContentPart::Text { text: system_text }],
    });

    // Previous chat history
    for (user_msg, assistant_msg) in chat_history {
        messages.push(Message {
            role: "user".to_string(),
            content: vec![ContentPart::Text {
                text: user_msg.clone(),
            }],
        });
        messages.push(Message {
            role: "assistant".to_string(),
            content: vec![ContentPart::Text {
                text: assistant_msg.clone(),
            }],
        });
    }

    // Current user message (possibly with screenshot)
    let mut user_content = Vec::new();
    user_content.push(ContentPart::Text {
        text: user_query.to_string(),
    });

    if let Some(b64) = screenshot_b64 {
        user_content.push(ContentPart::ImageUrl {
            image_url: ImageUrlData {
                url: format!("data:image/png;base64,{}", b64),
                detail: Some("high".to_string()),
            },
        });
    }

    messages.push(Message {
        role: "user".to_string(),
        content: user_content,
    });

    messages
}
