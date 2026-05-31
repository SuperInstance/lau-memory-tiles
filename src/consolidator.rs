use crate::{MemoryStore, MemoryTile, TileType};
use serde::{Deserialize, Serialize};

/// The consolidator — runs the "dream" phase to merge similar tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConsolidator {
    /// Minimum number of tiles to form a cluster for consolidation.
    pub min_cluster_size: usize,
    /// How much to reduce original tile importance after consolidation.
    pub consolidation_decay: f64,
}

impl Default for MemoryConsolidator {
    fn default() -> Self {
        Self {
            min_cluster_size: 2,
            consolidation_decay: 0.2,
        }
    }
}

impl MemoryConsolidator {
    pub fn new(min_cluster_size: usize, consolidation_decay: f64) -> Self {
        Self {
            min_cluster_size,
            consolidation_decay,
        }
    }

    /// Run consolidation on the store:
    /// 1. Find tiles with similar content (same tile_type + overlapping metadata)
    /// 2. Create a "Dream" tile that summarizes the cluster
    /// 3. Connect the dream tile to all members
    /// 4. Reduce importance of originals
    pub fn consolidate(&self, store: &mut MemoryStore) -> Vec<String> {
        let mut created_ids = Vec::new();
        let mut clustered: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Group tiles by (tile_type, metadata_key) for simple similarity
        let tile_ids: Vec<String> = store.tiles.keys().cloned().collect();

        for key in &tile_ids {
            if clustered.contains(key) {
                continue;
            }
            let Some(tile) = store.get_untouched(key) else {
                continue;
            };

            // Find similar tiles: same type + at least one shared metadata value
            let mut cluster_members: Vec<String> = vec![key.clone()];

            for other_id in &tile_ids {
                if other_id == key || clustered.contains(other_id) {
                    continue;
                }
                let Some(other) = store.get_untouched(other_id) else {
                    continue;
                };

                if self.are_similar(tile, other) {
                    cluster_members.push(other_id.clone());
                }
            }

            if cluster_members.len() >= self.min_cluster_size {
                // Create dream tile
                let dream_id = self.create_dream_tile(store, &cluster_members);
                created_ids.push(dream_id.clone());

                // Mark as clustered and decay originals
                for member_id in &cluster_members {
                    clustered.insert(member_id.clone());
                    if let Some(t) = store.tiles.get_mut(member_id) {
                        t.decay(self.consolidation_decay);
                        t.connect_to(&dream_id);
                    }
                }
            }
        }

        created_ids
    }

    /// Simple similarity check: same tile type and overlapping metadata.
    fn are_similar(&self, a: &MemoryTile, b: &MemoryTile) -> bool {
        if std::mem::discriminant(&a.tile_type) != std::mem::discriminant(&b.tile_type) {
            return false;
        }

        // Check metadata overlap
        let shared = a.metadata.keys().any(|k| b.metadata.contains_key(k) && a.metadata[k] == b.metadata[k]);

        // Also consider content similarity (simple substring check)
        let content_similar = a.content.contains(&b.content) || b.content.contains(&a.content);

        shared || content_similar
    }

    fn create_dream_tile(&self, store: &mut MemoryStore, members: &[String]) -> String {
        let first = store.get_untouched(&members[0]).unwrap();

        let summary = if members.len() <= 3 {
            members
                .iter()
                .filter_map(|id| store.get_untouched(id).map(|t| t.content.as_str()))
                .collect::<Vec<_>>()
                .join(" + ")
        } else {
            format!(
                "Consolidation of {} {} tiles",
                members.len(),
                first.tile_type
            )
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let avg_importance = members
            .iter()
            .filter_map(|id| store.get_untouched(id))
            .map(|t| t.importance)
            .sum::<f64>()
            / members.len() as f64;

        let mut dream = MemoryTile::new(&first.room_id, &first.agent_id, TileType::Dream, &summary)
            .with_id(&format!("dream_{}", now))
            .with_importance(avg_importance)
            .with_timestamps(now, now);

        // Connect dream to all members
        for member_id in members {
            dream.connect_to(member_id);
        }

        let dream_id = dream.id.clone();
        store.store(dream);
        dream_id
    }
}
