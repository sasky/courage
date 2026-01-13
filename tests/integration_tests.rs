//! Integration tests for the Courage application
//!
//! These tests verify the interaction between different modules.

use std::collections::HashMap;
use tempfile::TempDir;

// Re-export types we need for testing
use courage::people::{
    generate_person_markdown, read_all_people, update_person_file, write_person_file,
    PersonFrontmatter, PersonSections, Relationships,
};

/// Test the complete flow of creating a person file and reading it back
#[tokio::test]
async fn test_create_and_read_person() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    // Create a person
    let frontmatter = PersonFrontmatter {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        location: "Wellington".to_string(),
        work: "Software Engineer".to_string(),
        ..Default::default()
    };
    let relationships = Relationships {
        partner: "[[Jane Smith]]".to_string(),
        ..Default::default()
    };
    let sections = PersonSections {
        how_we_met: "Met at a tech conference".to_string(),
        ..Default::default()
    };

    let content = generate_person_markdown(&frontmatter, &relationships, &sections);
    write_person_file(people_dir, "John Smith", &content)
        .await
        .unwrap();

    // Read it back
    let context = read_all_people(people_dir).await.unwrap();

    assert_eq!(context.people.len(), 1);
    let person = &context.people[0];
    assert_eq!(person.frontmatter.first_name, "John");
    assert_eq!(person.frontmatter.last_name, "Smith");
    assert_eq!(person.frontmatter.location, "Wellington");
    assert_eq!(person.frontmatter.work, "Software Engineer");
}

/// Test creating multiple people and reading them all
#[tokio::test]
async fn test_read_multiple_people() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    // Create several people
    let people = vec![
        ("John Smith", "Engineer", "Wellington"),
        ("Jane Doe", "Designer", "Auckland"),
        ("Bob Jones", "Manager", "Christchurch"),
    ];

    for (name, work, location) in people {
        let parts: Vec<&str> = name.split_whitespace().collect();
        let frontmatter = PersonFrontmatter {
            first_name: parts[0].to_string(),
            last_name: parts[1].to_string(),
            work: work.to_string(),
            location: location.to_string(),
            ..Default::default()
        };
        let content =
            generate_person_markdown(&frontmatter, &Relationships::default(), &PersonSections::default());
        write_person_file(people_dir, name, &content).await.unwrap();
    }

    // Read them all
    let context = read_all_people(people_dir).await.unwrap();

    assert_eq!(context.people.len(), 3);

    // Check that all are present (order may vary)
    let names: Vec<String> = context
        .people
        .iter()
        .map(|p| format!("{} {}", p.frontmatter.first_name, p.frontmatter.last_name))
        .collect();

    assert!(names.contains(&"John Smith".to_string()));
    assert!(names.contains(&"Jane Doe".to_string()));
    assert!(names.contains(&"Bob Jones".to_string()));
}

/// Test updating an existing person file
#[tokio::test]
async fn test_update_person() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    // Create initial person
    let frontmatter = PersonFrontmatter {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        work: "Junior Engineer".to_string(),
        ..Default::default()
    };
    let content = generate_person_markdown(
        &frontmatter,
        &Relationships::default(),
        &PersonSections::default(),
    );
    write_person_file(people_dir, "John Smith", &content)
        .await
        .unwrap();

    // Update the person
    let mut field_updates = HashMap::new();
    field_updates.insert("work".to_string(), "Senior Engineer".to_string());

    let relationship_updates = Relationships {
        partner: "[[Jane Smith]]".to_string(),
        ..Default::default()
    };

    let section_updates = PersonSections {
        hobbies: "- Running".to_string(),
        ..Default::default()
    };

    update_person_file(
        people_dir,
        "John Smith",
        &field_updates,
        &relationship_updates,
        &section_updates,
    )
    .await
    .unwrap();

    // Read it back and verify updates
    let context = read_all_people(people_dir).await.unwrap();
    assert_eq!(context.people.len(), 1);

    let person = &context.people[0];
    assert_eq!(person.frontmatter.work, "Senior Engineer");
    assert!(person.content.contains("[[Jane Smith]]"));
    assert!(person.content.contains("- Running"));
}

/// Test that the people list is correctly formatted for LLM prompts
#[tokio::test]
async fn test_people_context_prompt_list() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    // Create a person with work and location
    let frontmatter = PersonFrontmatter {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        work: "Engineer at Google".to_string(),
        location: "Wellington".to_string(),
        ..Default::default()
    };
    let content = generate_person_markdown(
        &frontmatter,
        &Relationships::default(),
        &PersonSections::default(),
    );
    write_person_file(people_dir, "John Smith", &content)
        .await
        .unwrap();

    let context = read_all_people(people_dir).await.unwrap();
    let list = context.to_prompt_list();

    assert!(list.contains("John Smith"));
    assert!(list.contains("Engineer at Google"));
    assert!(list.contains("Wellington"));
}

/// Test reading from empty directory
#[tokio::test]
async fn test_read_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    let context = read_all_people(people_dir).await.unwrap();

    assert!(context.people.is_empty());
    assert_eq!(context.to_prompt_list(), "(No people in database yet)");
}

/// Test reading from non-existent directory
#[tokio::test]
async fn test_read_nonexistent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path().join("nonexistent");

    let context = read_all_people(&people_dir).await.unwrap();

    assert!(context.people.is_empty());
}

/// Test that non-markdown files are ignored
#[tokio::test]
async fn test_ignores_non_markdown_files() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    // Create a markdown file
    let frontmatter = PersonFrontmatter {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        ..Default::default()
    };
    let content = generate_person_markdown(
        &frontmatter,
        &Relationships::default(),
        &PersonSections::default(),
    );
    write_person_file(people_dir, "John Smith", &content)
        .await
        .unwrap();

    // Create a non-markdown file
    tokio::fs::write(people_dir.join("notes.txt"), "Some notes")
        .await
        .unwrap();
    tokio::fs::write(people_dir.join("data.json"), "{}")
        .await
        .unwrap();

    let context = read_all_people(people_dir).await.unwrap();

    // Should only have the markdown file
    assert_eq!(context.people.len(), 1);
    assert_eq!(context.people[0].frontmatter.first_name, "John");
}

/// Test bidirectional relationship scenario
#[tokio::test]
async fn test_bidirectional_relationships() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    // Create John with partner Jane
    let john_fm = PersonFrontmatter {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        ..Default::default()
    };
    let john_rel = Relationships {
        partner: "[[Jane Smith]]".to_string(),
        ..Default::default()
    };
    let john_content =
        generate_person_markdown(&john_fm, &john_rel, &PersonSections::default());
    write_person_file(people_dir, "John Smith", &john_content)
        .await
        .unwrap();

    // Create Jane with partner John
    let jane_fm = PersonFrontmatter {
        first_name: "Jane".to_string(),
        last_name: "Smith".to_string(),
        ..Default::default()
    };
    let jane_rel = Relationships {
        partner: "[[John Smith]]".to_string(),
        ..Default::default()
    };
    let jane_content =
        generate_person_markdown(&jane_fm, &jane_rel, &PersonSections::default());
    write_person_file(people_dir, "Jane Smith", &jane_content)
        .await
        .unwrap();

    // Read and verify
    let context = read_all_people(people_dir).await.unwrap();
    assert_eq!(context.people.len(), 2);

    for person in &context.people {
        if person.frontmatter.first_name == "John" {
            assert!(person.content.contains("[[Jane Smith]]"));
        } else if person.frontmatter.first_name == "Jane" {
            assert!(person.content.contains("[[John Smith]]"));
        }
    }
}

/// Test person with disambiguator
#[tokio::test]
async fn test_person_with_disambiguator() {
    let temp_dir = TempDir::new().unwrap();
    let people_dir = temp_dir.path();

    let frontmatter = PersonFrontmatter {
        first_name: "John".to_string(),
        last_name: "Smith".to_string(),
        disambiguator: "work".to_string(),
        ..Default::default()
    };
    let content = generate_person_markdown(
        &frontmatter,
        &Relationships::default(),
        &PersonSections::default(),
    );

    // The content should include the disambiguator in the heading
    assert!(content.contains("# John Smith (work)"));

    write_person_file(people_dir, "John Smith (work)", &content)
        .await
        .unwrap();

    let context = read_all_people(people_dir).await.unwrap();
    assert_eq!(context.people.len(), 1);
    assert_eq!(context.people[0].frontmatter.disambiguator, "work");
}
