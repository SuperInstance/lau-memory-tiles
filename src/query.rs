use crate::{MemoryStore, MemoryTile, TileType};
use serde::{Deserialize, Serialize};

/// A structured query over the memory store.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryQuery {
    pub room_id: Option<String>,
    pub agent_id: Option<String>,
    pub tile_type: Option<TileType>,
    pub min_importance: Option<f64>,
    pub time_range: Option<(u64, u64)>,
    pub content_contains: Option<String>,
    pub limit: Option<usize>,
}

impl MemoryQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn room_id(mut self, room_id: &str) -> Self {
        self.room_id = Some(room_id.to_string());
        self
    }

    pub fn agent_id(mut self, agent_id: &str) -> Self {
        self.agent_id = Some(agent_id.to_string());
        self
    }

    pub fn tile_type(mut self, tile_type: TileType) -> Self {
        self.tile_type = Some(tile_type);
        self
    }

    pub fn min_importance(mut self, min: f64) -> Self {
        self.min_importance = Some(min);
        self
    }

    pub fn time_range(mut self, start: u64, end: u64) -> Self {
        self.time_range = Some((start, end));
        self
    }

    pub fn content_contains(mut self, substring: &str) -> Self {
        self.content_contains = Some(substring.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Execute this query against a memory store.
    pub fn execute<'a>(&self, store: &'a MemoryStore) -> Vec<&'a MemoryTile> {
        let mut results: Vec<&MemoryTile> = store
            .tiles
            .values()
            .filter(|t| {
                if let Some(ref rid) = self.room_id {
                    if t.room_id != *rid {
                        return false;
                    }
                }
                if let Some(ref aid) = self.agent_id {
                    if t.agent_id != *aid {
                        return false;
                    }
                }
                if let Some(ref tt) = self.tile_type {
                    if &t.tile_type != tt {
                        return false;
                    }
                }
                if let Some(mi) = self.min_importance {
                    if t.importance < mi {
                        return false;
                    }
                }
                if let Some((start, end)) = self.time_range {
                    if t.created_at < start || t.created_at > end {
                        return false;
                    }
                }
                if let Some(ref needle) = self.content_contains {
                    if !t.content.contains(needle) {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Sort by importance descending
        results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        results
    }
}
