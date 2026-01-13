use serde::{Deserialize, Serialize};

/// Frontmatter fields for a person markdown file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonFrontmatter {
    pub first_name: String,
    pub last_name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub disambiguator: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub nickname: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub birthday: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub phone: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub email: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub location: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub work: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub created: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub updated: String,
}

/// Relationships extracted from a person file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Relationships {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parents: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub children: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub siblings: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub partner: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub friends: String,
}

/// Sections that can be updated in a person file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonSections {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub how_we_met: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub history: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub current_situation: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hobbies: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub favourite_media: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub other_notes: String,
}

/// A complete person record
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of public API
pub struct Person {
    /// Filename without extension (e.g., "John Smith")
    pub filename: String,
    /// Parsed frontmatter
    pub frontmatter: PersonFrontmatter,
    /// Relationships section
    pub relationships: Relationships,
    /// Full markdown content
    pub content: String,
}

impl Person {
    /// Get the display name for this person
    pub fn display_name(&self) -> String {
        let base = format!(
            "{} {}",
            self.frontmatter.first_name, self.frontmatter.last_name
        );
        if self.frontmatter.disambiguator.is_empty() {
            base
        } else {
            format!("{} ({})", base, self.frontmatter.disambiguator)
        }
    }

    /// Get a short summary for context (used in LLM prompts)
    pub fn summary(&self) -> String {
        let mut parts = vec![self.display_name()];
        if !self.frontmatter.work.is_empty() {
            parts.push(format!("- {}", self.frontmatter.work));
        }
        if !self.frontmatter.location.is_empty() {
            parts.push(format!("({})", self.frontmatter.location));
        }
        parts.join(" ")
    }
}

/// Context about all people for LLM processing
#[derive(Debug, Clone)]
pub struct PeopleContext {
    pub people: Vec<Person>,
}

impl PeopleContext {
    /// Generate a list of people for the LLM prompt
    pub fn to_prompt_list(&self) -> String {
        if self.people.is_empty() {
            return "(No people in database yet)".to_string();
        }

        self.people
            .iter()
            .map(|p| format!("- {}", p.summary()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_display_name_simple() {
        let person = Person {
            filename: "John Smith".to_string(),
            frontmatter: PersonFrontmatter {
                first_name: "John".to_string(),
                last_name: "Smith".to_string(),
                ..Default::default()
            },
            relationships: Relationships::default(),
            content: String::new(),
        };
        assert_eq!(person.display_name(), "John Smith");
    }

    #[test]
    fn test_person_display_name_with_disambiguator() {
        let person = Person {
            filename: "John Smith (work)".to_string(),
            frontmatter: PersonFrontmatter {
                first_name: "John".to_string(),
                last_name: "Smith".to_string(),
                disambiguator: "work".to_string(),
                ..Default::default()
            },
            relationships: Relationships::default(),
            content: String::new(),
        };
        assert_eq!(person.display_name(), "John Smith (work)");
    }

    #[test]
    fn test_person_summary_minimal() {
        let person = Person {
            filename: "John Smith".to_string(),
            frontmatter: PersonFrontmatter {
                first_name: "John".to_string(),
                last_name: "Smith".to_string(),
                ..Default::default()
            },
            relationships: Relationships::default(),
            content: String::new(),
        };
        assert_eq!(person.summary(), "John Smith");
    }

    #[test]
    fn test_person_summary_with_work_and_location() {
        let person = Person {
            filename: "John Smith".to_string(),
            frontmatter: PersonFrontmatter {
                first_name: "John".to_string(),
                last_name: "Smith".to_string(),
                work: "Software Engineer".to_string(),
                location: "Wellington".to_string(),
                ..Default::default()
            },
            relationships: Relationships::default(),
            content: String::new(),
        };
        assert_eq!(
            person.summary(),
            "John Smith - Software Engineer (Wellington)"
        );
    }

    #[test]
    fn test_people_context_empty() {
        let ctx = PeopleContext { people: vec![] };
        assert_eq!(ctx.to_prompt_list(), "(No people in database yet)");
    }

    #[test]
    fn test_people_context_with_people() {
        let ctx = PeopleContext {
            people: vec![
                Person {
                    filename: "John Smith".to_string(),
                    frontmatter: PersonFrontmatter {
                        first_name: "John".to_string(),
                        last_name: "Smith".to_string(),
                        work: "Engineer".to_string(),
                        ..Default::default()
                    },
                    relationships: Relationships::default(),
                    content: String::new(),
                },
                Person {
                    filename: "Jane Doe".to_string(),
                    frontmatter: PersonFrontmatter {
                        first_name: "Jane".to_string(),
                        last_name: "Doe".to_string(),
                        ..Default::default()
                    },
                    relationships: Relationships::default(),
                    content: String::new(),
                },
            ],
        };
        let list = ctx.to_prompt_list();
        assert!(list.contains("- John Smith - Engineer"));
        assert!(list.contains("- Jane Doe"));
    }

    #[test]
    fn test_frontmatter_default() {
        let fm = PersonFrontmatter::default();
        assert!(fm.first_name.is_empty());
        assert!(fm.last_name.is_empty());
        assert!(fm.tags.is_empty());
    }

    #[test]
    fn test_relationships_default() {
        let rel = Relationships::default();
        assert!(rel.parents.is_empty());
        assert!(rel.children.is_empty());
        assert!(rel.siblings.is_empty());
        assert!(rel.partner.is_empty());
        assert!(rel.friends.is_empty());
    }

    #[test]
    fn test_person_sections_default() {
        let sec = PersonSections::default();
        assert!(sec.how_we_met.is_empty());
        assert!(sec.history.is_empty());
        assert!(sec.current_situation.is_empty());
        assert!(sec.hobbies.is_empty());
        assert!(sec.favourite_media.is_empty());
        assert!(sec.other_notes.is_empty());
    }
}
