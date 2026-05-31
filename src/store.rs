use crate::{MemoryTile, TileType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages a collection of memory tiles with capacity limits, decay, and consolidation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    pub tiles: HashMap<String, MemoryTile>,
    pub max_tiles: usize,
}

impl MemoryStore {
    pub fn new(max_tiles: usize) -> Self {
        Self {
            tiles: HashMap::new(),
            max_tiles,
        }
    }

    /// Store a tile. If at capacity, evict the tile with the lowest importance.
    pub fn store(&mut self, mut tile: MemoryTile) {
        // Evict if at capacity (and this tile isn't already stored)
        if self.tiles.len() >= self.max_tiles && !self.tiles.contains_key(&tile.id) {
            self.evict_lowest();
        }
        tile.touch();
        self.tiles.insert(tile.id.clone(), tile);
    }

    /// Get a tile by ID. Auto-touches (reinforces) the tile.
    pub fn get(&mut self, id: &str) -> Option<&MemoryTile> {
        if let Some(tile) = self.tiles.get_mut(id) {
            tile.touch();
            tile.reinforce(0.01);
        }
        self.tiles.get(id)
    }

    /// Get a tile without touching it.
    pub fn get_untouched(&self, id: &str) -> Option<&MemoryTile> {
        self.tiles.get(id)
    }

    /// Query tiles with optional filters.
    pub fn query(
        &self,
        room_id: Option<&str>,
        tile_type: Option<&TileType>,
        min_importance: Option<f64>,
    ) -> Vec<&MemoryTile> {
        self.tiles
            .values()
            .filter(|t| {
                if let Some(rid) = room_id {
                    if t.room_id != rid {
                        return false;
                    }
                }
                if let Some(tt) = tile_type {
                    if &t.tile_type != tt {
                        return false;
                    }
                }
                if let Some(mi) = min_importance {
                    if t.importance < mi {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Get the `n` most recent tiles (by created_at descending).
    pub fn recent(&self, n: usize) -> Vec<&MemoryTile> {
        let mut tiles: Vec<&MemoryTile> = self.tiles.values().collect();
        tiles.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        tiles.into_iter().take(n).collect()
    }

    /// Get the `n` most important tiles.
    pub fn important(&self, n: usize) -> Vec<&MemoryTile> {
        let mut tiles: Vec<&MemoryTile> = self.tiles.values().collect();
        tiles.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap_or(std::cmp::Ordering::Equal));
        tiles.into_iter().take(n).collect()
    }

    /// Decay all tiles by the given rate.
    pub fn decay_all(&mut self, rate: f64) {
        for tile in self.tiles.values_mut() {
            tile.decay(rate);
        }
    }

    /// Consolidation placeholder — see MemoryConsolidator for full logic.
    pub fn consolidate(&mut self) {
        // Basic consolidation: decay low-access tiles more
        for tile in self.tiles.values_mut() {
            if tile.access_count == 0 {
                tile.decay(0.05);
            }
        }
    }

    /// Remove tiles below the given importance threshold.
    pub fn prune(&mut self, min_importance: f64) {
        self.tiles.retain(|_, t| t.importance >= min_importance);
    }

    /// Number of tiles in the store.
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Memory pressure: tile_count / max_tiles.
    pub fn memory_pressure(&self) -> f64 {
        if self.max_tiles == 0 {
            return 1.0;
        }
        self.tiles.len() as f64 / self.max_tiles as f64
    }

    /// Evict the tile with the lowest importance.
    fn evict_lowest(&mut self) {
        if self.tiles.is_empty() {
            return;
        }
        let lowest_id = self
            .tiles
            .iter()
            .min_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id.clone());
        if let Some(id) = lowest_id {
            self.tiles.remove(&id);
        }
    }

    /// Remove a tile by ID.
    pub fn remove(&mut self, id: &str) -> Option<MemoryTile> {
        self.tiles.remove(id)
    }
}
