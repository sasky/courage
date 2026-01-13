use anyhow::{Context, Result};
use chrono::Local;
use std::path::Path;

use super::types::{PersonFrontmatter, Relationships, PersonSections};
use crate::utils::markdown::relationship_pattern;

/// Generate a new person markdown file
pub fn generate_person_markdown(
    frontmatter: &PersonFrontmatter,
    relationships: &Relationships,
    sections: &PersonSections,
) -> String {
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut fm = frontmatter.clone();
    if fm.created.is_empty() {
        fm.created = today.clone();
    }
    fm.updated = today.clone();
    if !fm.tags.contains(&"person".to_string()) {
        fm.tags.push("person".to_string());
    }

    let display_name = if fm.disambiguator.is_empty() {
        format!("{} {}", fm.first_name, fm.last_name)
    } else {
        format!("{} {} ({})", fm.first_name, fm.last_name, fm.disambiguator)
    };

    format!(
        r#"---
first_name: {}
last_name: {}
disambiguator: {}
nickname: {}
birthday: {}
phone: {}
email: {}
location: {}
work: {}
tags: [{}]
created: {}
updated: {}
---

# {}

## Relationships
**Parents:** {}
**Children:** {}
**Siblings:** {}
**Partner:** {}
**Friends:** {}

## How We Met
{}

## History
{}

## Current Situation
{}

## Hobbies & Passions
{}

## Favourite Media
{}

## Other Notes
{}
"#,
        fm.first_name,
        fm.last_name,
        fm.disambiguator,
        fm.nickname,
        fm.birthday,
        fm.phone,
        fm.email,
        fm.location,
        fm.work,
        fm.tags.join(", "),
        fm.created,
        fm.updated,
        display_name,
        relationships.parents,
        relationships.children,
        relationships.siblings,
        relationships.partner,
        relationships.friends,
        sections.how_we_met,
        sections.history,
        sections.current_situation,
        sections.hobbies,
        sections.favourite_media,
        sections.other_notes,
    )
}

/// Write a person file to disk
pub async fn write_person_file(
    people_dir: &Path,
    filename: &str,
    content: &str,
) -> Result<()> {
    // Ensure directory exists
    tokio::fs::create_dir_all(people_dir).await?;

    let path = people_dir.join(format!("{}.md", filename));
    tokio::fs::write(&path, content)
        .await
        .context("Failed to write person file")?;

    tracing::info!("Wrote person file: {:?}", path);
    Ok(())
}

/// Update an existing person file by merging changes
pub async fn update_person_file(
    people_dir: &Path,
    filename: &str,
    field_updates: &std::collections::HashMap<String, String>,
    relationship_updates: &Relationships,
    section_updates: &PersonSections,
) -> Result<String> {
    let path = people_dir.join(format!("{}.md", filename));
    let existing = tokio::fs::read_to_string(&path)
        .await
        .context("Failed to read existing file")?;

    let updated = merge_person_content(&existing, field_updates, relationship_updates, section_updates)?;

    tokio::fs::write(&path, &updated)
        .await
        .context("Failed to write updated file")?;

    tracing::info!("Updated person file: {:?}", path);
    Ok(updated)
}

/// Merge updates into existing markdown content
fn merge_person_content(
    existing: &str,
    field_updates: &std::collections::HashMap<String, String>,
    relationship_updates: &Relationships,
    section_updates: &PersonSections,
) -> Result<String> {
    let mut content = existing.to_string();
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Update frontmatter fields
    for (key, value) in field_updates {
        if !value.is_empty() {
            content = update_frontmatter_field(&content, key, value);
        }
    }

    // Always update the 'updated' timestamp
    content = update_frontmatter_field(&content, "updated", &today);

    // Merge relationships (append new links)
    content = merge_relationship(&content, "Parents", &relationship_updates.parents);
    content = merge_relationship(&content, "Children", &relationship_updates.children);
    content = merge_relationship(&content, "Siblings", &relationship_updates.siblings);
    content = merge_relationship(&content, "Partner", &relationship_updates.partner);
    content = merge_relationship(&content, "Friends", &relationship_updates.friends);

    // Append to sections
    content = append_to_section(&content, "How We Met", &section_updates.how_we_met);
    content = append_to_section(&content, "History", &section_updates.history);
    content = append_to_section(&content, "Current Situation", &section_updates.current_situation);
    content = append_to_section(&content, "Hobbies & Passions", &section_updates.hobbies);
    content = append_to_section(&content, "Favourite Media", &section_updates.favourite_media);
    content = append_to_section(&content, "Other Notes", &section_updates.other_notes);

    Ok(content)
}

/// Update a frontmatter field
fn update_frontmatter_field(content: &str, key: &str, value: &str) -> String {
    let pattern = format!(r"(?m)^{}:.*$", regex::escape(key));
    let re = regex::Regex::new(&pattern).unwrap();

    if re.is_match(content) {
        re.replace(content, format!("{}: {}", key, value)).to_string()
    } else {
        // Field doesn't exist, add it before the closing ---
        content.replacen("---\n\n", &format!("{}: {}\n---\n\n", key, value), 1)
    }
}

/// Merge a relationship value (append if new links provided)
fn merge_relationship(content: &str, label: &str, new_value: &str) -> String {
    if new_value.is_empty() {
        return content.to_string();
    }

    // Use shared regex pattern for relationship extraction
    let re = relationship_pattern(label);

    if let Some(caps) = re.captures(content) {
        let existing = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");

        let merged = if existing.is_empty() {
            new_value.to_string()
        } else if existing.contains(new_value) {
            // Already has this link
            existing.to_string()
        } else {
            format!("{}, {}", existing, new_value)
        };

        re.replace(content, format!("**{}:** {}", label, merged)).to_string()
    } else {
        content.to_string()
    }
}

/// Append content to a section
fn append_to_section(content: &str, section_name: &str, new_content: &str) -> String {
    if new_content.is_empty() {
        return content.to_string();
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let dated_content = new_content.replace("{{DATE}}", &today);

    // Find the section header
    let section_pattern = format!(r"(## {})\n", regex::escape(section_name));
    let re = regex::Regex::new(&section_pattern).unwrap();

    if let Some(mat) = re.find(content) {
        let insert_pos = mat.end();

        // Find the next section or end of file
        let rest = &content[insert_pos..];
        let next_section = rest.find("\n## ").unwrap_or(rest.len());
        let existing_content = rest[..next_section].trim();

        let updated_section = if existing_content.is_empty() {
            format!("{}\n\n", dated_content)
        } else {
            format!("{}\n\n{}\n\n", existing_content, dated_content)
        };

        format!(
            "{}## {}\n{}{}",
            &content[..mat.start()],
            section_name,
            updated_section,
            &rest[next_section..]
        )
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_frontmatter_field_existing() {
        let content = "---\nfirst_name: John\nlast_name: Doe\n---\n\n# John Doe";
        let updated = update_frontmatter_field(content, "last_name", "Smith");
        assert!(updated.contains("last_name: Smith"));
        assert!(!updated.contains("last_name: Doe"));
    }

    #[test]
    fn test_update_frontmatter_field_preserves_other_fields() {
        let content = "---\nfirst_name: John\nlast_name: Doe\nwork: Engineer\n---\n\n# John Doe";
        let updated = update_frontmatter_field(content, "last_name", "Smith");
        assert!(updated.contains("first_name: John"));
        assert!(updated.contains("work: Engineer"));
    }

    #[test]
    fn test_merge_relationship_append() {
        let content = "**Parents:** [[Mom]]\n**Children:** \n";
        let updated = merge_relationship(content, "Parents", "[[Dad]]");
        assert!(updated.contains("**Parents:** [[Mom]], [[Dad]]"));
    }

    #[test]
    fn test_merge_relationship_empty_existing() {
        let content = "**Parents:** \n**Children:** \n";
        let updated = merge_relationship(content, "Parents", "[[Mom]]");
        assert!(updated.contains("**Parents:** [[Mom]]"));
    }

    #[test]
    fn test_merge_relationship_already_exists() {
        let content = "**Parents:** [[Mom]], [[Dad]]\n**Children:** \n";
        let updated = merge_relationship(content, "Parents", "[[Mom]]");
        // Should not duplicate
        assert_eq!(updated.matches("[[Mom]]").count(), 1);
    }

    #[test]
    fn test_merge_relationship_empty_new_value() {
        let content = "**Parents:** [[Mom]]\n**Children:** \n";
        let updated = merge_relationship(content, "Parents", "");
        assert_eq!(updated, content);
    }

    #[test]
    fn test_append_to_section_empty() {
        let content = "## How We Met\n\n## History\n";
        let updated = append_to_section(content, "How We Met", "Met at a conference");
        assert!(updated.contains("Met at a conference"));
    }

    #[test]
    fn test_append_to_section_with_existing() {
        let content = "## How We Met\nExisting content here\n\n## History\n";
        let updated = append_to_section(content, "How We Met", "New info");
        assert!(updated.contains("Existing content here"));
        assert!(updated.contains("New info"));
    }

    #[test]
    fn test_append_to_section_replaces_date_placeholder() {
        let content = "## Hobbies & Passions\n\n## Other Notes\n";
        let updated = append_to_section(content, "Hobbies & Passions", "Running (added {{DATE}})");
        assert!(updated.contains("Running (added"));
        assert!(!updated.contains("{{DATE}}"));
    }

    #[test]
    fn test_generate_person_markdown_basic() {
        let fm = PersonFrontmatter {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            location: "Wellington".to_string(),
            ..Default::default()
        };
        let rel = Relationships::default();
        let sec = PersonSections::default();

        let md = generate_person_markdown(&fm, &rel, &sec);

        assert!(md.contains("first_name: John"));
        assert!(md.contains("last_name: Smith"));
        assert!(md.contains("location: Wellington"));
        assert!(md.contains("# John Smith"));
        assert!(md.contains("## Relationships"));
        assert!(md.contains("## How We Met"));
    }

    #[test]
    fn test_generate_person_markdown_with_disambiguator() {
        let fm = PersonFrontmatter {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            disambiguator: "work".to_string(),
            ..Default::default()
        };
        let rel = Relationships::default();
        let sec = PersonSections::default();

        let md = generate_person_markdown(&fm, &rel, &sec);

        assert!(md.contains("# John Smith (work)"));
    }

    #[test]
    fn test_generate_person_markdown_with_relationships() {
        let fm = PersonFrontmatter {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            ..Default::default()
        };
        let rel = Relationships {
            partner: "[[Jane Smith]]".to_string(),
            friends: "[[Bob Jones]], [[Alice Wong]]".to_string(),
            ..Default::default()
        };
        let sec = PersonSections::default();

        let md = generate_person_markdown(&fm, &rel, &sec);

        assert!(md.contains("**Partner:** [[Jane Smith]]"));
        assert!(md.contains("**Friends:** [[Bob Jones]], [[Alice Wong]]"));
    }

    #[test]
    fn test_generate_person_markdown_with_sections() {
        let fm = PersonFrontmatter {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            ..Default::default()
        };
        let rel = Relationships::default();
        let sec = PersonSections {
            how_we_met: "Met at a tech conference".to_string(),
            hobbies: "- Running\n- Photography".to_string(),
            ..Default::default()
        };

        let md = generate_person_markdown(&fm, &rel, &sec);

        assert!(md.contains("Met at a tech conference"));
        assert!(md.contains("- Running"));
        assert!(md.contains("- Photography"));
    }

    #[test]
    fn test_generate_person_markdown_adds_person_tag() {
        let fm = PersonFrontmatter {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            tags: vec!["family".to_string()],
            ..Default::default()
        };
        let rel = Relationships::default();
        let sec = PersonSections::default();

        let md = generate_person_markdown(&fm, &rel, &sec);

        assert!(md.contains("tags: [family, person]"));
    }

    #[test]
    fn test_generate_person_markdown_sets_dates() {
        let fm = PersonFrontmatter {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            ..Default::default()
        };
        let rel = Relationships::default();
        let sec = PersonSections::default();

        let md = generate_person_markdown(&fm, &rel, &sec);

        // Should have created and updated dates in YYYY-MM-DD format
        assert!(md.contains("created: 20"));
        assert!(md.contains("updated: 20"));
    }
}
