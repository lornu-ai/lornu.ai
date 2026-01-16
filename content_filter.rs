use anyhow::Result;

/// Defines the outcome of a content filter check.
#[derive(Debug, PartialEq)]
pub enum FilterAction {
    /// The content is considered safe and can be passed through.
    Allow,
    /// The content is flagged as potentially unsafe and should be quarantined.
    Flag,
}

/// A trait for content safety filters.
pub trait ContentFilter: Send + Sync {
    /// Checks a piece of text and determines the appropriate action.
    fn check_content(&self, content: &str) -> Result<FilterAction>;
}

/// A basic, example content filter that flags content containing specific keywords.
/// In a real-world scenario, this would be replaced by a more robust service.
pub struct BasicContentFilter;

impl ContentFilter for BasicContentFilter {
    fn check_content(&self, content: &str) -> Result<FilterAction> {
        // This is a placeholder for a real content safety check.
        // We're simulating a check for a known trigger word.
        if content.contains("unsafe-content") {
            // In a real system, this would trigger a quarantining process.
            println!("🚨 Content flagged for review: contains 'unsafe-content'");
            Ok(FilterAction::Flag)
        } else {
            Ok(FilterAction::Allow)
        }
    }
}