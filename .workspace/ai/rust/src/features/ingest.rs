use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use crate::providers::Provider;
use crate::cache::CacheManager;
use tokio::fs;
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedChunk {
    pub chunk: CodeChunk,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub file: String,
    pub name: String,
    pub snippet_type: String, // function, class, method
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub language: String,
}

pub struct IngestService {
    provider: Arc<dyn Provider + Send + Sync>,
    cache: Arc<CacheManager>,
}

impl IngestService {
    pub fn new(provider: Arc<dyn Provider + Send + Sync>, cache: Arc<CacheManager>) -> Self {
        Self { provider, cache }
    }

    pub async fn ingest_workspace(&self, workspace: &Path) -> Result<Vec<EmbeddedChunk>> {
        let files = self.discover_files(workspace).await?;
        let store_dir = workspace.join(".workspace/ai/models/embeddings");
        tokio::fs::create_dir_all(&store_dir).await?;

        let mut all_chunks = Vec::new();
        let mut all_snippets = Vec::new();

        for file in &files {
            let content = tokio::fs::read_to_string(file).await?;
            let lang = file.extension().and_then(|e| e.to_str()).unwrap_or("");
            let chunks = self.chunk_content(&content, lang, file)?;
            let snippets = self.extract_snippets(&content, lang, file);
            all_snippets.extend(snippets);

            for chunk in &chunks {
                let embedded = self.embed_chunk(chunk).await?;
                all_chunks.push(embedded);
            }
        }

        // Save embedding index
        let path = store_dir.join("index.json");
        let json = serde_json::to_string_pretty(&all_chunks)?;
        tokio::fs::write(path, json).await?;

        // Save snippets to docs/snippets/
        if !all_snippets.is_empty() {
            let snippets_dir = workspace.join(".workspace/docs/snippets");
            tokio::fs::create_dir_all(&snippets_dir).await?;
            let snippet_path = snippets_dir.join("index.json");
            let snippet_json = serde_json::to_string_pretty(&all_snippets)?;
            tokio::fs::write(snippet_path, snippet_json).await?;
        }

        Ok(all_chunks)
    }

    pub async fn search(&self, query: &str, workspace: &Path, top_k: usize) -> Result<Vec<EmbeddedChunk>> {
        let store_path = workspace.join(".workspace/ai/models/embeddings/index.json");
        if !store_path.exists() {
            return Err(anyhow!("No index found. Run 'polyglid-ai ingest' first."));
        }

        let json = tokio::fs::read_to_string(store_path).await?;
        let all: Vec<EmbeddedChunk> = serde_json::from_str(&json)?;

        let query_emb = self.provider.embed(query).await?;

        let mut scored: Vec<(f32, &EmbeddedChunk)> = all.iter()
            .map(|c| {
                let sim = cosine_similarity(&query_emb, &c.embedding);
                (sim, c)
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored.into_iter().take(top_k).map(|(_, c)| c.clone()).collect())
    }

    async fn discover_files(&self, workspace: &Path) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();
        let mut stack = vec![workspace.join("projects")];

        while let Some(dir) = stack.pop() {
            if !dir.is_dir() { continue; }
            let mut read_dir = tokio::fs::read_dir(&dir).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.') && name != "node_modules" && name != "target" {
                        stack.push(path);
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "h" | "cpp" | "hpp" | "toml" | "json" | "yaml" | "yml" | "md" => {
                            files.push(path);
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(files)
    }

    fn chunk_content(&self, content: &str, language: &str, file: &Path) -> Result<Vec<CodeChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let chunk_size = 100;
        let overlap = 10;
        let rel_path = file.to_string_lossy().to_string();

        for start in (0..lines.len()).step_by(chunk_size - overlap) {
            let end = (start + chunk_size).min(lines.len());
            let chunk_lines: Vec<&str> = lines[start..end].to_vec();
            let chunk_content = chunk_lines.join("\n");

            chunks.push(CodeChunk {
                file: rel_path.clone(),
                start_line: start + 1,
                end_line: end,
                content: chunk_content,
                language: language.to_string(),
            });

            if end == lines.len() { break; }
        }

        Ok(chunks)
    }

    async fn embed_chunk(&self, chunk: &CodeChunk) -> Result<EmbeddedChunk> {
        let embedding = self.provider.embed(&chunk.content).await?;
        Ok(EmbeddedChunk {
            chunk: chunk.clone(),
            embedding,
        })
    }

    fn extract_snippets(&self, content: &str, language: &str, file: &Path) -> Vec<Snippet> {
        let mut snippets = Vec::new();
        let rel_path = file.to_string_lossy().to_string();
        let lines: Vec<&str> = content.lines().collect();
        match language {
            "rs" => {
                let re = Regex::new(r"^\s*(pub\s+)?(unsafe\s+)?fn\s+(\w+)").unwrap();
                for (i, line) in lines.iter().enumerate() {
                    if let Some(caps) = re.captures(line) {
                        let name = caps.get(3).map(|m| m.as_str()).unwrap_or("unknown").to_string();
                        let end = (i + 20).min(lines.len());
                        let snippet_content = lines[i..end].join("\n");
                        snippets.push(Snippet {
                            file: rel_path.clone(),
                            name,
                            snippet_type: "function".to_string(),
                            start_line: i + 1,
                            end_line: end,
                            content: snippet_content,
                            language: language.to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
        snippets
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}

pub fn check_index_exists(workspace: &std::path::Path) -> bool {
    workspace.join(".workspace/ai/models/embeddings/index.json").exists()
}
