//! `OllamaAgent` — a single-agent wrapper around the Ollama OpenAI-compat client.
//!
//! Each agent holds:
//! - A reference to the `OpenAiCompatClient` (configured for `http://localhost:11434/v1`)
//! - Its Ollama model name (stripped of the `ollama/` prefix for the request)
//! - Its role system prompt
//! - A growing `Vec<InputMessage>` conversation history (within a phase)
//!
//! Call `agent.turn(user_text)` to send one message and get the assistant reply back.
//! Call `agent.reset()` to clear the history between phases.

use api::{
    InputMessage, MessageRequest, OpenAiCompatClient, OpenAiCompatConfig, OutputContentBlock,
};

/// Maximum tokens to request from Ollama per turn.
const MAX_TOKENS: u32 = 4096;

/// Strip the `ollama/` prefix the CLI uses internally so the raw model
/// name goes to Ollama (e.g. `ollama/qwen2.5-coder:14b` → `qwen2.5-coder:14b`).
fn strip_ollama_prefix(model: &str) -> &str {
    model.strip_prefix("ollama/").unwrap_or(model)
}

pub struct OllamaAgent {
    client: OpenAiCompatClient,
    /// Raw model name sent to Ollama (no `ollama/` prefix).
    model: String,
    /// Role system prompt (static string from `roles` module).
    system_prompt: &'static str,
    /// Conversation history accumulated within a phase.
    history: Vec<InputMessage>,
}

impl OllamaAgent {
    /// Create a new agent.  `model` may include or omit the `ollama/` prefix.
    #[must_use]
    pub fn new(model: &str, system_prompt: &'static str) -> Self {
        let config = OpenAiCompatConfig::ollama();
        // Ollama doesn't need an API key; we pass a placeholder.
        let client = OpenAiCompatClient::new("ollama", config);
        Self {
            client,
            model: strip_ollama_prefix(model).to_string(),
            system_prompt,
            history: Vec::new(),
        }
    }

    /// Send `user_text` as a user turn, append both messages to history,
    /// and return the assistant's response text.
    ///
    /// # Errors
    /// Returns an `ApiError` if the Ollama request fails.
    pub async fn turn(&mut self, user_text: &str) -> Result<String, api::ApiError> {
        // Append the user message
        self.history.push(InputMessage::user_text(user_text));

        let request = MessageRequest {
            model: self.model.clone(),
            max_tokens: MAX_TOKENS,
            messages: self.history.clone(),
            system: Some(self.system_prompt.to_string()),
            tools: None,
            tool_choice: None,
            stream: false,
        };

        let response = self.client.send_message(&request).await?;

        // Extract text from the response
        let text: String = response
            .content
            .iter()
            .filter_map(|block| {
                if let OutputContentBlock::Text { text } = block {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        // Append the assistant reply to history for multi-turn conversations
        self.history.push(InputMessage {
            role: "assistant".to_string(),
            content: vec![api::InputContentBlock::Text { text: text.clone() }],
        });

        Ok(text)
    }

    /// Clear conversation history. Call between pipeline phases.
    pub fn reset(&mut self) {
        self.history.clear();
    }

    /// Agent identifier label (for logging).
    #[must_use]
    pub fn label(&self) -> &str {
        // Derived from the system prompt's first heading line
        if self.system_prompt.contains("Planner") {
            "A"
        } else if self.system_prompt.contains("Plan Reviewer") {
            "B"
        } else if self.system_prompt.contains("Coder") {
            "C"
        } else if self.system_prompt.contains("Reviewer and Tester") {
            "D"
        } else {
            "S"
        }
    }
}
