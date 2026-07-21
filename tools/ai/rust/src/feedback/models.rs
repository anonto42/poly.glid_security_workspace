use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionStatus {
    Pending,
    Accepted,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: String,
    pub timestamp: String,
    pub category: String,        // code_generation, test_generation, code_review, security, dependency, build_optimization
    pub input: String,           // what was asked
    pub output: String,          // what was suggested
    pub status: PredictionStatus,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Prediction {
    pub fn new(id: String, category: String, input: String, output: String) -> Self {
        Self {
            id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            category,
            input,
            output,
            status: PredictionStatus::Pending,
            metadata: std::collections::HashMap::new(),
        }
    }
}
