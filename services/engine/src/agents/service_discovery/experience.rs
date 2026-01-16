//! Experience Store
//!
//! Episodic memory for the AI-SRE agent to learn from past
//! incidents and remediations.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::info;

/// A single experience record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Description of what happened
    pub description: String,
    /// Whether the action was successful
    pub success: bool,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Strength/confidence (increases with repeated successful patterns)
    pub strength: i32,
}

/// Episodic memory store for learning from past incidents
///
/// Implements the "Learning-as-it-Works" architecture from Issue #119:
/// - Stores every incident and remediation outcome
/// - Allows querying similar past incidents
/// - Reinforces successful patterns
pub struct ExperienceStore {
    experiences: VecDeque<Experience>,
    max_size: usize,
}

impl ExperienceStore {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            experiences: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Record a new experience
    pub fn record(&mut self, description: &str, success: bool) {
        // Check if similar experience exists
        if let Some(existing) = self
            .experiences
            .iter_mut()
            .find(|e| e.description == description)
        {
            // Reinforce: +1 for success, -1 for failure
            if success {
                existing.strength += 1;
                info!(
                    "Reinforced experience (strength: {}): {}",
                    existing.strength, description
                );
            } else {
                existing.strength -= 1;
                info!(
                    "Weakened experience (strength: {}): {}",
                    existing.strength, description
                );
            }
            existing.timestamp = chrono::Utc::now();
            return;
        }

        // Add new experience
        let experience = Experience {
            description: description.to_string(),
            success,
            timestamp: chrono::Utc::now(),
            strength: if success { 1 } else { -1 },
        };

        info!(
            "New experience recorded (success: {}): {}",
            success, description
        );

        // Remove oldest if at capacity
        if self.experiences.len() >= self.max_size {
            self.experiences.pop_front();
        }

        self.experiences.push_back(experience);
    }

    /// Query similar past experiences
    pub fn query(&self, keywords: &[&str]) -> Vec<&Experience> {
        self.experiences
            .iter()
            .filter(|e| {
                keywords
                    .iter()
                    .any(|k| e.description.to_lowercase().contains(&k.to_lowercase()))
            })
            .collect()
    }

    /// Get experiences with positive strength (successful patterns)
    pub fn successful_patterns(&self) -> Vec<&Experience> {
        self.experiences
            .iter()
            .filter(|e| e.strength > 0)
            .collect()
    }

    /// Get experiences with negative strength (failed patterns to avoid)
    pub fn failed_patterns(&self) -> Vec<&Experience> {
        self.experiences
            .iter()
            .filter(|e| e.strength < 0)
            .collect()
    }

    /// Check if a pattern has been tried before and its outcome
    pub fn has_pattern(&self, description: &str) -> Option<&Experience> {
        self.experiences
            .iter()
            .find(|e| e.description.contains(description))
    }

    /// Get statistics: (total_experiences, success_rate)
    pub fn stats(&self) -> (usize, f64) {
        let total = self.experiences.len();
        if total == 0 {
            return (0, 0.0);
        }

        let successful = self.experiences.iter().filter(|e| e.strength > 0).count();
        (total, successful as f64 / total as f64)
    }

    /// Export experiences for persistence
    pub fn export(&self) -> Vec<Experience> {
        self.experiences.iter().cloned().collect()
    }

    /// Import experiences from persistence
    pub fn import(&mut self, experiences: Vec<Experience>) {
        for exp in experiences {
            if self.experiences.len() >= self.max_size {
                self.experiences.pop_front();
            }
            self.experiences.push_back(exp);
        }
    }
}

impl Default for ExperienceStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query() {
        let mut store = ExperienceStore::new();

        store.record("GCP SSL failure fixed by disabling proxy", true);
        store.record("Azure failover successful", true);
        store.record("AWS timeout error", false);

        let ssl_results = store.query(&["SSL", "proxy"]);
        assert_eq!(ssl_results.len(), 1);
        assert!(ssl_results[0].success);

        let azure_results = store.query(&["Azure"]);
        assert_eq!(azure_results.len(), 1);
    }

    #[test]
    fn test_reinforcement() {
        let mut store = ExperienceStore::new();

        // Record same pattern multiple times
        store.record("Proxy disable fixes SSL", true);
        store.record("Proxy disable fixes SSL", true);
        store.record("Proxy disable fixes SSL", true);

        let patterns = store.successful_patterns();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].strength, 3);

        // Record failure
        store.record("Proxy disable fixes SSL", false);
        assert_eq!(store.successful_patterns()[0].strength, 2);
    }

    #[test]
    fn test_stats() {
        let mut store = ExperienceStore::new();

        store.record("Success 1", true);
        store.record("Success 2", true);
        store.record("Failure 1", false);

        let (total, rate) = store.stats();
        assert_eq!(total, 3);
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_capacity_limit() {
        let mut store = ExperienceStore::with_capacity(3);

        store.record("First", true);
        store.record("Second", true);
        store.record("Third", true);
        store.record("Fourth", true);

        assert_eq!(store.experiences.len(), 3);
        assert!(store.query(&["First"]).is_empty()); // First should be evicted
        assert!(!store.query(&["Fourth"]).is_empty());
    }
}
