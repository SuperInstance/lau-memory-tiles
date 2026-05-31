use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// A graph of connections between memory tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraph {
    /// (from_id, to_id, strength)
    pub edges: Vec<(String, String, f64)>,
}

impl MemoryGraph {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Connect two tiles with a given strength.
    pub fn connect(&mut self, from: &str, to: &str, strength: f64) {
        // Update existing edge or add new one
        if let Some(edge) = self.edges.iter_mut().find(|(f, t, _)| f == from && t == to) {
            edge.2 = strength;
        } else {
            self.edges.push((from.to_string(), to.to_string(), strength));
        }
    }

    /// Remove a connection between two tiles.
    pub fn disconnect(&mut self, from: &str, to: &str) {
        self.edges.retain(|(f, t, _)| !(f == from && t == to));
    }

    /// Get all connections from/to a tile.
    pub fn connections_of(&self, tile_id: &str) -> Vec<(&String, &f64)> {
        self.edges
            .iter()
            .filter(|(f, t, _)| f == tile_id || t == tile_id)
            .map(|(f, t, s)| {
                if f == tile_id {
                    (t, s)
                } else {
                    (f, s)
                }
            })
            .collect()
    }

    /// Find the strongest path between two tiles using BFS with strength weighting.
    /// Returns the sequence of tile IDs forming the path, or None if no path exists.
    pub fn strongest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        // Build adjacency list
        let mut adj: HashMap<&str, Vec<(&str, f64)>> = HashMap::new();
        for (f, t, s) in &self.edges {
            adj.entry(f.as_str()).or_default().push((t.as_str(), *s));
            adj.entry(t.as_str()).or_default().push((f.as_str(), *s));
        }

        // BFS tracking best cumulative strength to each node
        let mut best: HashMap<&str, (f64, Vec<String>)> = HashMap::new();
        let mut queue: VecDeque<(&str, f64, Vec<String>)> = VecDeque::new();

        queue.push_back((from, 1.0, vec![from.to_string()]));
        best.insert(from, (1.0, vec![from.to_string()]));

        while let Some((current, cum_strength, path)) = queue.pop_front() {
            if let Some(neighbors) = adj.get(current) {
                for &(neighbor, edge_strength) in neighbors {
                    let new_strength = cum_strength * edge_strength;
                    let new_path = {
                        let mut p = path.clone();
                        p.push(neighbor.to_string());
                        p
                    };

                    let should_visit = match best.get(neighbor) {
                        Some((existing, _)) => new_strength > *existing,
                        None => true,
                    };

                    if should_visit && !path.contains(&neighbor.to_string()) {
                        best.insert(neighbor, (new_strength, new_path.clone()));
                        if neighbor == to {
                            // Continue to find potentially stronger paths
                            queue.push_back((neighbor, new_strength, new_path));
                        } else {
                            queue.push_back((neighbor, new_strength, new_path));
                        }
                    }
                }
            }
        }

        best.get(to).map(|(_, path)| path.clone())
    }

    /// Calculate average connection strength within a group of tiles.
    pub fn cluster(&self, tile_ids: &[String]) -> f64 {
        if tile_ids.len() < 2 {
            return 0.0;
        }

        let id_set: HashSet<&str> = tile_ids.iter().map(|s| s.as_str()).collect();
        let mut total_strength = 0.0;
        let mut count = 0;

        for (f, t, s) in &self.edges {
            if id_set.contains(f.as_str()) && id_set.contains(t.as_str()) {
                total_strength += s;
                count += 1;
            }
        }

        if count == 0 {
            0.0
        } else {
            total_strength / count as f64
        }
    }

    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for MemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}
