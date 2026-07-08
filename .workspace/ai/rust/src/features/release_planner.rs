use std::path::Path;
use anyhow::{Result, anyhow};
use tokio::fs;
use serde::Serialize;

#[derive(Serialize)]
pub struct ReleaseManifest {
    pub project: String,
    pub version: String,
    pub language: String,
    pub manifest_yaml: String,
}

pub struct ReleasePlanner;

impl ReleasePlanner {
    pub async fn analyze(workspace_path: &Path) -> Result<Vec<ReleaseManifest>> {
        let projects = workspace_path.join("projects");
        if !projects.exists() {
            return Err(anyhow!("No projects/ directory found"));
        }

        let mut manifests = Vec::new();
        let mut rd = fs::read_dir(&projects).await?;

        while let Some(entry) = rd.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() { continue; }

            // Detect Rust project
            let cargo = path.join("Cargo.toml");
            if cargo.exists() {
                let text = fs::read_to_string(&cargo).await?;
                if let Ok(config) = toml::from_str::<toml::Value>(&text) {
                    let name = config.get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");
                    let version = config.get("package")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.1.0");
                    manifests.push(ReleaseManifest {
                        project: name.to_string(),
                        version: version.to_string(),
                        language: "rust".to_string(),
                        manifest_yaml: format!(r#"apiVersion: v1
kind: Deployment
metadata:
  name: {name}
spec:
  replicas: 1
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      containers:
        - name: {name}
          image: polyglid/{name}:{version}
          ports:
            - containerPort: 8080
"#, name = name, version = version),
                    });
                }
            }

            // Detect Node project
            let pkg = path.join("package.json");
            if pkg.exists() {
                let text = fs::read_to_string(&pkg).await?;
                if let Ok(config) = serde_json::from_str::<serde_json::Value>(&text) {
                    let name = config.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let version = config.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.0");
                    manifests.push(ReleaseManifest {
                        project: name.to_string(),
                        version: version.to_string(),
                        language: "node".to_string(),
                        manifest_yaml: format!(r#"apiVersion: v1
kind: Deployment
metadata:
  name: {name}
spec:
  replicas: 2
  selector:
    matchLabels:
      app: {name}
  template:
    metadata:
      labels:
        app: {name}
    spec:
      containers:
        - name: {name}
          image: polyglid/{name}:{version}
          ports:
            - containerPort: 3000
"#, name = name, version = version),
                    });
                }
            }
        }

        Ok(manifests)
    }
}
