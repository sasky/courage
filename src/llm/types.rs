use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::people::{Relationships, PersonSections};

/// Response from Claude after processing a transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub status: ResponseStatus,
    pub message: String,
    #[serde(default)]
    pub actions: Vec<PersonAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ready,
    ClarificationNeeded,
    Error,
}

/// An action to create or update a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonAction {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    pub filename: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub relationships: ActionRelationships,
    #[serde(default)]
    pub sections: ActionSections,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Create,
    Update,
}

/// Relationships in action format (string values)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionRelationships {
    #[serde(default)]
    pub parents: Option<String>,
    #[serde(default)]
    pub children: Option<String>,
    #[serde(default)]
    pub siblings: Option<String>,
    #[serde(default)]
    pub partner: Option<String>,
    #[serde(default)]
    pub friends: Option<String>,
}

impl From<ActionRelationships> for Relationships {
    fn from(ar: ActionRelationships) -> Self {
        Relationships {
            parents: ar.parents.unwrap_or_default(),
            children: ar.children.unwrap_or_default(),
            siblings: ar.siblings.unwrap_or_default(),
            partner: ar.partner.unwrap_or_default(),
            friends: ar.friends.unwrap_or_default(),
        }
    }
}

/// Sections in action format
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActionSections {
    #[serde(default)]
    pub how_we_met: Option<String>,
    #[serde(default)]
    pub history: Option<String>,
    #[serde(default)]
    pub current_situation: Option<String>,
    #[serde(default)]
    pub hobbies: Option<String>,
    #[serde(default)]
    pub favourite_media: Option<String>,
    #[serde(default)]
    pub other_notes: Option<String>,
}

impl From<ActionSections> for PersonSections {
    fn from(as_: ActionSections) -> Self {
        PersonSections {
            how_we_met: as_.how_we_met.unwrap_or_default(),
            history: as_.history.unwrap_or_default(),
            current_situation: as_.current_situation.unwrap_or_default(),
            hobbies: as_.hobbies.unwrap_or_default(),
            favourite_media: as_.favourite_media.unwrap_or_default(),
            other_notes: as_.other_notes.unwrap_or_default(),
        }
    }
}

/// Message for Claude API
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

/// Request body for Claude API
#[derive(Debug, Clone, Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeMessage>,
}

/// Response from Claude API
#[derive(Debug, Clone, Deserialize)]
pub struct ClaudeApiResponse {
    pub content: Vec<ClaudeContent>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields are part of API response structure
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ready_response() {
        let json = r#"{
            "status": "ready",
            "message": "Creating John Smith",
            "actions": [
                {
                    "type": "create",
                    "filename": "John Smith.md",
                    "fields": {
                        "first_name": "John",
                        "last_name": "Smith"
                    },
                    "relationships": {},
                    "sections": {}
                }
            ]
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::Ready);
        assert_eq!(response.message, "Creating John Smith");
        assert_eq!(response.actions.len(), 1);
        assert_eq!(response.actions[0].action_type, ActionType::Create);
        assert_eq!(response.actions[0].filename, "John Smith.md");
    }

    #[test]
    fn test_parse_clarification_response() {
        let json = r#"{
            "status": "clarification_needed",
            "message": "Which John do you mean?",
            "actions": []
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::ClarificationNeeded);
        assert_eq!(response.message, "Which John do you mean?");
        assert!(response.actions.is_empty());
    }

    #[test]
    fn test_parse_error_response() {
        let json = r#"{
            "status": "error",
            "message": "Could not parse transcript",
            "actions": []
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::Error);
    }

    #[test]
    fn test_parse_update_action() {
        let json = r#"{
            "status": "ready",
            "message": "Updating John",
            "actions": [
                {
                    "type": "update",
                    "filename": "John Smith.md",
                    "fields": {
                        "work": "Senior Engineer"
                    },
                    "relationships": {
                        "partner": "[[Jane Smith]]"
                    },
                    "sections": {
                        "hobbies": "Marathon training"
                    }
                }
            ]
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions[0].action_type, ActionType::Update);
        assert_eq!(
            response.actions[0].fields.get("work"),
            Some(&"Senior Engineer".to_string())
        );
        assert_eq!(
            response.actions[0].relationships.partner,
            Some("[[Jane Smith]]".to_string())
        );
        assert_eq!(
            response.actions[0].sections.hobbies,
            Some("Marathon training".to_string())
        );
    }

    #[test]
    fn test_parse_multiple_actions() {
        let json = r#"{
            "status": "ready",
            "message": "Creating relationship",
            "actions": [
                {
                    "type": "create",
                    "filename": "Mike.md",
                    "fields": {"first_name": "Mike", "last_name": ""},
                    "relationships": {"partner": "[[Sarah]]"},
                    "sections": {}
                },
                {
                    "type": "update",
                    "filename": "Sarah.md",
                    "fields": {},
                    "relationships": {"partner": "[[Mike]]"},
                    "sections": {}
                }
            ]
        }"#;

        let response: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.actions.len(), 2);
        assert_eq!(response.actions[0].action_type, ActionType::Create);
        assert_eq!(response.actions[1].action_type, ActionType::Update);
    }

    #[test]
    fn test_action_relationships_to_relationships() {
        let ar = ActionRelationships {
            parents: Some("[[Mom]], [[Dad]]".to_string()),
            partner: Some("[[Spouse]]".to_string()),
            children: None,
            siblings: None,
            friends: None,
        };

        let rel: Relationships = ar.into();
        assert_eq!(rel.parents, "[[Mom]], [[Dad]]");
        assert_eq!(rel.partner, "[[Spouse]]");
        assert_eq!(rel.children, "");
        assert_eq!(rel.siblings, "");
        assert_eq!(rel.friends, "");
    }

    #[test]
    fn test_action_sections_to_person_sections() {
        let as_ = ActionSections {
            how_we_met: Some("At a conference".to_string()),
            hobbies: Some("Running".to_string()),
            history: None,
            current_situation: None,
            favourite_media: None,
            other_notes: None,
        };

        let sec: PersonSections = as_.into();
        assert_eq!(sec.how_we_met, "At a conference");
        assert_eq!(sec.hobbies, "Running");
        assert_eq!(sec.history, "");
    }

    #[test]
    fn test_serialize_llm_response() {
        let response = LlmResponse {
            status: ResponseStatus::Ready,
            message: "Done".to_string(),
            actions: vec![],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"ready\""));
        assert!(json.contains("\"message\":\"Done\""));
    }

    #[test]
    fn test_claude_request_serialization() {
        let request = ClaudeRequest {
            model: "claude-3-5-haiku-20241022".to_string(),
            max_tokens: 4096,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-3-5-haiku"));
        assert!(json.contains("4096"));
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }
}
