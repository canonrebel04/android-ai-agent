use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub skill: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug)]
pub enum ParseError {
    NoToolCall,
    InvalidJson,
}

pub fn parse(response: &str) -> Result<AgentAction, ParseError> {
    // Try OpenAI-style tool_calls first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
        if let Some(tool_calls) = json["tool_calls"].as_array() {
            if let Some(first) = tool_calls.first() {
                let function = &first["function"];
                return Ok(AgentAction {
                    skill: function["name"].as_str().unwrap_or("unknown").to_string(),
                    parameters: function["arguments"].clone(),
                });
            }
        }
        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
            return Ok(AgentAction {
                skill: name.to_string(),
                parameters: json
                    .get("parameters")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            });
        }
    }

    // Try markdown ```json block
    if let Some(start) = response.find("```json") {
        let after_start = &response[start + 7..];
        if let Some(end) = after_start.find("```") {
            let json_str = &after_start[..end].trim();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(name) = parsed
                    .get("skill")
                    .or_else(|| parsed.get("name"))
                    .and_then(|v| v.as_str())
                {
                    return Ok(AgentAction {
                        skill: name.to_string(),
                        parameters: parsed
                            .get("parameters")
                            .or(parsed.get("args"))
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }

    Err(ParseError::NoToolCall)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_tool_call() {
        let response =
            r#"{"tool_calls": [{"function": {"name": "web_search", "arguments": {"query": "rust lang"}}}]}"#;
        let action = parse(response).unwrap();
        assert_eq!(action.skill, "web_search");
        assert_eq!(action.parameters["query"], "rust lang");
    }

    #[test]
    fn test_parse_markdown_json() {
        let response = "Here's the action:\n```json\n{\"skill\": \"open_app\", \"parameters\": {\"name\": \"calculator\"}}\n```";
        let action = parse(response).unwrap();
        assert_eq!(action.skill, "open_app");
        assert_eq!(action.parameters["name"], "calculator");
    }

    #[test]
    fn test_no_tool_call() {
        let response = "I don't know how to do that.";
        assert!(parse(response).is_err());
    }
}
