pub mod parser;
pub mod types;
pub mod writer;

pub use parser::read_all_people;
#[allow(unused_imports)] // Re-exports for public API
pub use types::{PeopleContext, Person, PersonFrontmatter, PersonSections, Relationships};
pub use writer::{generate_person_markdown, update_person_file, write_person_file};
