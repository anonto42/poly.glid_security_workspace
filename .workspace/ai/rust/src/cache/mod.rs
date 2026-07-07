//! Cache Management for AI Operations

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use moka::sync::Cache;
use dashmap::DashMap;
use chrono::{DateTime, Utc};

use crate::core::models::Suggestion;

/// Cache entry with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T, ttl_seconds: u64) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
            ttl_seconds,
        }
    }
    
    pub fn is_valid(&self) -> bool {
        let elapsed = Utc::now() - self.timestamp;
        elapsed.num_seconds() < self.ttl_seconds as i64
    }
}

/// Cache Manager
pub struct CacheManager {
    /// In-memory cache
    memory_cache: Cache<String, Vec<u8>>,
    
    /// Disk cache path
    disk_cache_path: PathBuf,
    
    /// Cache statistics
    stats: DashMap<String, CacheStats>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size_bytes: u64,
    pub last_cleanup: DateTime<Utc>,
}

impl CacheManager {
    /// Create new cache manager
    pub fn new(workspace_path: &Path) -> Result<Self> {
        let disk_cache_path = workspace_path
            .join(".workspace")
            .join("ai")
            .join("cache");
        
        std::fs::create_dir_all(&disk_cache_path)?;
        
        // Create memory cache with 1000 entries, 1 hour TTL
        let memory_cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(3600))
            .build();
        
        Ok(Self {
            memory_cache,
            disk_cache_path,
            stats: DashMap::new(),
        })
    }
    
    /// Store suggestions in cache
    pub async fn store_suggestions(&self, suggestions: &[Suggestion]) -> Result<()> {
        let key = "suggestions".to_string();
        let entry = CacheEntry::new(suggestions.to_vec(), 3600);
        let data = bincode::serialize(&entry)?;
        
        // Store in memory
        self.memory_cache.insert(key.clone(), data.clone());
        
        // Store on disk
        let disk_path = self.disk_cache_path.join("suggestions.cache");
        tokio::fs::write(disk_path, data).await?;
        
        // Update stats
        self.update_stats(&key, data.len() as u64).await?;
        
        Ok(())
    }
    
    /// Get suggestions from cache
    pub async fn get_suggestions(&self, limit: usize) -> Result<Option<Vec<Suggestion>>> {
        let key = "suggestions".to_string();
        
        // Try memory cache first
        if let Some(data) = self.memory_cache.get(&key) {
            if let Ok(entry) = bincode::deserialize::<CacheEntry<Vec<Suggestion>>>(&data) {
                if entry.is_valid() {
                    self.record_hit(&key).await?;
                    return Ok(Some(entry.data.into_iter().take(limit).collect()));
                }
            }
        }
        
        // Try disk cache
        let disk_path = self.disk_cache_path.join("suggestions.cache");
        if disk_path.exists() {
            if let Ok(data) = tokio::fs::read(&disk_path).await {
                if let Ok(entry) = bincode::deserialize::<CacheEntry<Vec<Suggestion>>>(&data) {
                    if entry.is_valid() {
                        // Store back in memory
                        let serialized = bincode::serialize(&entry)?;
                        self.memory_cache.insert(key.clone(), serialized);
                        
                        self.record_hit(&key).await?;
                        return Ok(Some(entry.data.into_iter().take(limit).collect()));
                    }
                }
            }
        }
        
        self.record_miss(&key).await?;
        Ok(None)
    }
    
    /// Store analysis in cache
    pub async fn store_analysis(&self, analysis: &crate::core::models::Analysis) -> Result<()> {
        let key = format!("analysis_{}", analysis.timestamp.timestamp());
        let entry = CacheEntry::new(analysis.clone(), 3600);
        let data = bincode::serialize(&entry)?;
        
        // Store on disk
        let disk_path = self.disk_cache_path.join(format!("analysis_{}.cache", key));
        tokio::fs::write(disk_path, data).await?;
        
        Ok(())
    }
    
    /// Get analysis from cache
    pub async fn get_analysis(&self, timestamp: i64) -> Result<Option<crate::core::models::Analysis>> {
        let key = format!("analysis_{}", timestamp);
        let disk_path = self.disk_cache_path.join(format!("analysis_{}.cache", key));
        
        if disk_path.exists() {
            if let Ok(data) = tokio::fs::read(&disk_path).await {
                if let Ok(entry) = bincode::deserialize::<CacheEntry<crate::core::models::Analysis>>(&data) {
                    if entry.is_valid() {
                        self.record_hit(&key).await?;
                        return Ok(Some(entry.data));
                    }
                }
            }
        }
        
        self.record_miss(&key).await?;
        Ok(None)
    }
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.memory_cache.invalidate_all();
        
        // Remove disk cache files
        for entry in std::fs::read_dir(&self.disk_cache_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("cache") {
                std::fs::remove_file(path)?;
            }
        }
        
        Ok(())
    }
    
    /// Update cache statistics
    async fn update_stats(&self, key: &str, size: u64) -> Result<()> {
        let mut stats = self.stats.entry(key.to_string()).or_insert(CacheStats {
            hits: 0,
            misses: 0,
            size_bytes: 0,
            last_cleanup: Utc::now(),
        });
        
        stats.size_bytes += size;
        
        Ok(())
    }
    
    /// Record cache hit
    async fn record_hit(&self, key: &str) -> Result<()> {
        let mut stats = self.stats.entry(key.to_string()).or_insert(CacheStats {
            hits: 0,
            misses: 0,
            size_bytes: 0,
            last_cleanup: Utc::now(),
        });
        
        stats.hits += 1;
        
        Ok(())
    }
    
    /// Record cache miss
    async fn record_miss(&self, key: &str) -> Result<()> {
        let mut stats = self.stats.entry(key.to_string()).or_insert(CacheStats {
            hits: 0,
            misses: 0,
            size_bytes: 0,
            last_cleanup: Utc::now(),
        });
        
        stats.misses += 1;
        
        Ok(())
    }
    
    /// Get cache statistics
    pub fn get_stats(&self) -> HashMap<String, CacheStats> {
        self.stats.iter().map(|entry| {
            (entry.key().clone(), entry.value().clone())
        }).collect()
    }
}
