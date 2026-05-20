pub mod diff;
pub mod errors;
pub mod frontmatter;
pub mod ids;
pub mod time;

pub fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
