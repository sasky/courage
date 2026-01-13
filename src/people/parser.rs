use anyhow::{Context, Result};
use std::path::Path;

use super::types::{Person, PersonFrontmatter, PeopleContext, Relationships};
use crate::utils::markdown::extract_labeled_value;

/// Read all person files from a directory
pub async fn read_all_people(dir: &Path) -> Result<PeopleContext> {
    let mut people = Vec::new();

    if !dir.exists() {
        tracing::warn!("People directory does not exist: {:?}", dir);
        return Ok(PeopleContext { people });
    }

    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md") {
            match read_person(&path).await {
                Ok(person) => people.push(person),
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(PeopleContext { people })
}

/// Read and parse a single person file
pub async fn read_person(path: &Path) -> Result<Person> {
    let content = tokio::fs::read_to_string(path)
        .await
        .context("Failed to read file")?;

    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    parse_person_content(&filename, &content)
}

/// Parse person content from markdown string
pub fn parse_person_content(filename: &str, content: &str) -> Result<Person> {
    // Parse frontmatter using gray_matter
    let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
    let parsed = matter.parse(content);

    let frontmatter: PersonFrontmatter = if let Some(data) = parsed.data {
        match data {
            gray_matter::Pod::Hash(map) => {
                // Convert Pod to serde_yaml::Value then deserialize
                let yaml_str = pod_to_yaml_string(&gray_matter::Pod::Hash(map))?;
                serde_yaml::from_str(&yaml_str).unwrap_or_default()
            }
            _ => PersonFrontmatter::default(),
        }
    } else {
        PersonFrontmatter::default()
    };

    // Parse relationships from content
    let relationships = parse_relationships(&parsed.content);

    Ok(Person {
        filename: filename.to_string(),
        frontmatter,
        relationships,
        content: content.to_string(),
    })
}

/// Convert gray_matter Pod to YAML string for serde parsing
fn pod_to_yaml_string(pod: &gray_matter::Pod) -> Result<String> {
    match pod {
        gray_matter::Pod::Hash(map) => {
            let mut yaml = String::new();
            for (key, value) in map {
                yaml.push_str(&format!("{}: {}\n", key, pod_value_to_string(value)));
            }
            Ok(yaml)
        }
        _ => Ok(String::new()),
    }
}

fn pod_value_to_string(pod: &gray_matter::Pod) -> String {
    match pod {
        gray_matter::Pod::String(s) => {
            if s.contains(':') || s.contains('#') || s.starts_with(' ') {
                format!("\"{}\"", s.replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        gray_matter::Pod::Integer(i) => i.to_string(),
        gray_matter::Pod::Float(f) => f.to_string(),
        gray_matter::Pod::Boolean(b) => b.to_string(),
        gray_matter::Pod::Array(arr) => {
            let items: Vec<String> = arr.iter().map(pod_value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        gray_matter::Pod::Hash(_) => "{}".to_string(),
        gray_matter::Pod::Null => "".to_string(),
    }
}

/// Parse relationships section from markdown content
fn parse_relationships(content: &str) -> Relationships {
    let mut relationships = Relationships::default();

    // Look for ## Relationships section
    if let Some(rel_start) = content.find("## Relationships") {
        let rel_section = &content[rel_start..];
        let rel_end = rel_section[16..] // Skip "## Relationships"
            .find("\n## ")
            .map(|i| i + 16)
            .unwrap_or(rel_section.len());
        let rel_text = &rel_section[..rel_end];

        // Extract each relationship type using shared utility
        relationships.parents = extract_labeled_value(rel_text, "Parents");
        relationships.children = extract_labeled_value(rel_text, "Children");
        relationships.siblings = extract_labeled_value(rel_text, "Siblings");
        relationships.partner = extract_labeled_value(rel_text, "Partner");
        relationships.friends = extract_labeled_value(rel_text, "Friends");
    }

    relationships
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relationships() {
        let content = r#"## Relationships
**Parents:** [[John Smith]], [[Mary Smith]]
**Children:** 
**Siblings:** [[Sarah Smith]]
**Partner:** [[Jane Doe]]
**Friends:** [[Bob Jones]]

## How We Met
"#;
        let rel = parse_relationships(content);
        assert_eq!(rel.parents, "[[John Smith]], [[Mary Smith]]");
        assert_eq!(rel.siblings, "[[Sarah Smith]]");
        assert_eq!(rel.partner, "[[Jane Doe]]");
        assert_eq!(rel.friends, "[[Bob Jones]]");
        assert_eq!(rel.children, "");
    }

    #[test]
    fn test_extract_labeled_value() {
        let text = "**Parents:** [[Mom]], [[Dad]]\n**Children:** \n";
        assert_eq!(extract_labeled_value(text, "Parents"), "[[Mom]], [[Dad]]");
        assert_eq!(extract_labeled_value(text, "Children"), "");
    }
}
