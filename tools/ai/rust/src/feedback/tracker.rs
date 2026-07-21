use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use crate::feedback::models::{Prediction, PredictionStatus};
use tokio::fs;

pub struct FeedbackTracker {
    store_dir: PathBuf,
}

impl FeedbackTracker {
    pub fn new(workspace: &Path) -> Self {
        Self { store_dir: workspace.join(".workspace").join("ai").join("predictions") }
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.store_dir.join(format!("{}.json", id))
    }

    async fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.store_dir).await?;
        Ok(())
    }

    pub async fn save_prediction(&self, prediction: &Prediction) -> Result<()> {
        self.ensure_dir().await?;
        let json = serde_json::to_string_pretty(prediction)?;
        fs::write(self.path_for(&prediction.id), &json).await?;
        Ok(())
    }

    pub async fn update_status(&self, id: &str, status: PredictionStatus) -> Result<Prediction> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(anyhow!("Prediction {} not found", id));
        }
        let json = fs::read_to_string(&path).await?;
        let mut prediction: Prediction = serde_json::from_str(&json)?;
        prediction.status = status;
        let updated = serde_json::to_string_pretty(&prediction)?;
        fs::write(&path, &updated).await?;
        Ok(prediction)
    }

    pub async fn list_predictions(&self, category: Option<&str>) -> Result<Vec<Prediction>> {
        self.ensure_dir().await?;
        let mut predictions = Vec::new();
        let mut rd = fs::read_dir(&self.store_dir).await?;
        while let Some(entry) = rd.next_entry().await? {
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                let json = fs::read_to_string(entry.path()).await?;
                if let Ok(p) = serde_json::from_str::<Prediction>(&json) {
                    if let Some(cat) = category {
                        if p.category == cat {
                            predictions.push(p);
                        }
                    } else {
                        predictions.push(p);
                    }
                }
            }
        }
        predictions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(predictions)
    }

    pub async fn get_prediction(&self, id: &str) -> Result<Prediction> {
        let path = self.path_for(id);
        if !path.exists() {
            return Err(anyhow!("Prediction {} not found", id));
        }
        let json = fs::read_to_string(&path).await?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn generate_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        format!("pred_{:x}", nanos)
    }
}
