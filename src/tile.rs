use crate::TileType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single memory entry — one tile in an agent's memory mosaic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTile {
    pub id: String,
    pub room_id: String,
    pub agent_id: String,
    pub tile_type: TileType,
    pub content: String,
    /// Importance score in [0.0, 1.0].
    pub importance: f64,
    /// How many times this tile has been accessed.
    pub access_count: u64,
    /// Unix epoch millis when created.
    pub created_at: u64,
    /// Unix epoch millis when last accessed.
    pub last_accessed: u64,
    /// Optional semantic embedding vector.
    pub embeddings: Vec<f64>,
    /// IDs of related tiles.
    pub connections: Vec<String>,
    /// Arbitrary key-value metadata.
    pub metadata: HashMap<String, String>,
}

impl MemoryTile {
    /// Create a new tile. `id` is auto-generated. `created_at` and `last_accessed` are set to now.
    pub fn new(room_id: &str, agent_id: &str, tile_type: TileType, content: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: format!("tile_{}", now),
            room_id: room_id.to_string(),
            agent_id: agent_id.to_string(),
            tile_type,
            content: content.to_string(),
            importance: 0.5,
            access_count: 0,
            created_at: now,
            last_accessed: now,
            embeddings: Vec::new(),
            connections: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a tile with a specific id (useful for testing).
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Create a tile with a specific importance.
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Create a tile with specific timestamps.
    pub fn with_timestamps(mut self, created_at: u64, last_accessed: u64) -> Self {
        self.created_at = created_at;
        self.last_accessed = last_accessed;
        self
    }

    /// Create a tile with metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Create a tile with embeddings.
    pub fn with_embeddings(mut self, embeddings: Vec<f64>) -> Self {
        self.embeddings = embeddings;
        self
    }

    /// Touch this tile: increment access count and update last_accessed.
    pub fn touch(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.access_count += 1;
        self.last_accessed = now;
    }

    /// Touch with a specific timestamp (for deterministic testing).
    pub fn touch_at(&mut self, now: u64) {
        self.access_count += 1;
        self.last_accessed = now;
    }

    /// Decay importance over time. importance = max(0, importance - rate).
    pub fn decay(&mut self, rate: f64) {
        self.importance = (self.importance - rate).max(0.0);
    }

    /// Reinforce importance (increase when accessed).
    pub fn reinforce(&mut self, amount: f64) {
        self.importance = (self.importance + amount).min(1.0);
    }

    /// Connect this tile to another by ID.
    pub fn connect_to(&mut self, other_id: &str) {
        if !self.connections.contains(&other_id.to_string()) {
            self.connections.push(other_id.to_string());
        }
    }

    /// Age of this tile in milliseconds.
    pub fn age(&self, now: u64) -> u64 {
        now.saturating_sub(self.created_at)
    }

    /// Returns true if the tile was created within `threshold_ms` of `now`.
    pub fn is_fresh(&self, now: u64, threshold_ms: u64) -> bool {
        self.age(now) <= threshold_ms
    }
}
