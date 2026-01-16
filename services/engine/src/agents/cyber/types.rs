use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamInsight {
    pub email: String,
    pub description: String,
    pub recommended_role: String,
    pub unused_permissions: Vec<String>,
}

impl IamInsight {
    pub fn has_unused_permissions(&self) -> bool {
        !self.unused_permissions.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IamCorrection {
    pub sa_email: String,
    pub old_role: String,
    pub new_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountInfo {
    pub email: String,
    pub current_role: String,
    pub last_authenticated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShrinkPattern {
    pub original_role: String,
    pub suggested_role: String,
    pub reasoning: String,
}
