# lau-memory-tiles

**Memory tiles for AI agents — observations, actions, thoughts, decisions, dreams, and more.**

A lightweight, structured memory system where each memory is a "tile" — a typed, timestamped, importance-weighted record of a cognitive event. Tiles live in a capacity-bounded store, decay over time, get reinforced on access, and can be consolidated during "dream" phases into higher-order abstractions.

---

## What This Does

This crate provides the memory substrate for an AI agent:

1. **TileType** — 9 typed memory categories: Observation, Action, Thought, Decision, Dream, Error, Signal, Response, and Custom.
2. **MemoryTile** — individual memory entries with importance scoring, access counting, embedding vectors, connections to other tiles, and arbitrary metadata.
3. **MemoryStore** — capacity-bounded tile storage with LRU-like eviction (lowest importance first), decay, reinforcement on access, pruning, and memory-pressure tracking.
4. **MemoryGraph** — weighted connection graph between tiles. Supports finding the strongest path between two tiles via BFS.
5. **MemoryQuery** — fluent builder for filtered, sorted queries over the store (by room, agent, type, importance, time range, content substring).
6. **MemoryConsolidator** — "dream phase" engine that groups similar tiles, creates Dream tiles summarizing clusters, connects them to originals, and decays the source tiles.

Everything is pure Rust, zero dependencies beyond `serde` + `serde_json`, and fully serializable.

---

## Key Idea

> **Agent memory should work like human memory.** Recent events are vivid. Frequently recalled memories get reinforced. Unused memories decay. During idle time (dreaming), similar memories consolidate into abstract knowledge. The result is a living, evolving memory mosaic — not a flat log.

The importance dynamics:

- **Decay:** `importance -= rate` (clamped at 0)
- **Reinforce:** `importance += amount` (clamped at 1) — triggered on access
- **Consolidation:** cluster similar tiles → create Dream tile at average importance → decay originals

---

## Install

```toml
[dependencies]
lau-memory-tiles = "0.1"
```

Requires Rust 2021 edition. Dependencies: `serde`, `serde_json`.

---

## Quick Start

```rust
use lau_memory_tiles::*;

fn main() {
    // Create a memory store with capacity 1000
    let mut store = MemoryStore::new(1000);

    // Store some tiles
    let obs = MemoryTile::new("room-1", "agent-1", TileType::Observation, "User asked about the weather")
        .with_importance(0.7)
        .with_metadata("topic", "weather");
    store.store(obs);

    let action = MemoryTile::new("room-1", "agent-1", TileType::Action, "Fetched weather from API")
        .with_importance(0.6)
        .with_metadata("topic", "weather");
    store.store(action);

    let thought = MemoryTile::new("room-1", "agent-1", TileType::Thought, "User seems interested in outdoor plans")
        .with_importance(0.5)
        .with_metadata("topic", "weather");
    store.store(thought);

    // Query: find all weather-related tiles
    let query = MemoryQuery::new()
        .room_id("room-1")
        .content_contains("weather")
        .min_importance(0.3);
    let results = query.execute(&store);
    println!("Found {} weather tiles", results.len());

    // Consolidate similar memories into dreams
    let consolidator = MemoryConsolidator::new(2, 0.2);
    let dream_ids = consolidator.consolidate(&mut store);
    println!("Created {} dream tiles", dream_ids.len());

    // Memory pressure
    println!("Memory pressure: {:.1}%", store.memory_pressure() * 100.0);
}
```

---

## API Reference

### `TileType` — memory category

```rust
pub enum TileType {
    Observation,  // what happened
    Action,       // what was done
    Thought,      // internal reasoning
    Decision,     // a choice made
    Dream,        // consolidation artifact
    Error,        // something went wrong
    Signal,       // sensor/port data
    Response,     // output to user/agent
    Custom(String), // free-form
}
```

Methods: `is_observable()` (external events), `is_internal()` (cognitive events), `Display`.

### `MemoryTile` — a single memory entry

| method | description |
|---|---|
| `new(room_id, agent_id, tile_type, content)` | create tile with auto-generated ID and current timestamp |
| `with_id(id)` | set specific ID |
| `with_importance(f64)` | set importance [0, 1] |
| `with_timestamps(created, accessed)` | set specific timestamps |
| `with_metadata(key, value)` | add key-value metadata |
| `with_embeddings(vec)` | attach embedding vector |
| `touch()` / `touch_at(now)` | increment access count, update last_accessed |
| `decay(rate)` | importance -= rate (clamped at 0) |
| `reinforce(amount)` | importance += amount (clamped at 1) |
| `connect_to(other_id)` | add bidirectional connection |
| `age(now)` | age in milliseconds |
| `is_fresh(now, threshold)` | created within threshold? |

Fields: `id`, `room_id`, `agent_id`, `tile_type`, `content`, `importance`, `access_count`, `created_at`, `last_accessed`, `embeddings`, `connections`, `metadata`.

### `MemoryStore` — capacity-bounded tile storage

| method | description |
|---|---|
| `new(max_tiles)` | create with capacity |
| `store(tile)` | insert tile; evict lowest-importance if at capacity |
| `get(id)` | retrieve + touch + reinforce (+0.01 importance) |
| `get_untouched(id)` | retrieve without side effects |
| `query(room_id, tile_type, min_importance)` | simple filtered query |
| `recent(n)` | n most recent by created_at |
| `important(n)` | n highest importance |
| `decay_all(rate)` | decay all tiles |
| `prune(min_importance)` | remove tiles below threshold |
| `consolidate()` | basic: extra decay for untouched tiles |
| `remove(id)` | remove by ID |
| `tile_count()` | current count |
| `memory_pressure()` | count / capacity ratio |

### `MemoryQuery` — fluent query builder

```rust
let results = MemoryQuery::new()
    .room_id("room-1")
    .agent_id("agent-1")
    .tile_type(TileType::Observation)
    .min_importance(0.5)
    .time_range(start_ms, end_ms)
    .content_contains("keyword")
    .limit(10)
    .execute(&store);
```

Returns `Vec<&MemoryTile>` sorted by importance descending.

### `MemoryGraph` — weighted connection graph

| method | description |
|---|---|
| `connect(from, to, strength)` | add/update weighted edge |
| `disconnect(from, to)` | remove edge |
| `connections_of(tile_id)` | all edges touching a tile |
| `strongest_path(from, to)` | BFS finding highest cumulative-strength path |
| `cluster(&[ids])` | average edge strength within a group |

### `MemoryConsolidator` — dream-phase engine

| method | description |
|---|---|
| `new(min_cluster_size, consolidation_decay)` | configure |
| `consolidate(&mut store) → Vec<String>` | find similar tiles, create Dream tiles, decay originals |

Similarity: same TileType + (overlapping metadata values OR one content contains the other).

Consolidation process:
1. Group tiles by type and similarity
2. For clusters ≥ min_cluster_size: create a Dream tile with summarized content
3. Connect dream to all cluster members
4. Decay original tiles by consolidation_decay

---

## How It Works

### Memory Lifecycle

```
Create → Store (touch) → Access (touch + reinforce) → Decay → Consolidate → Dream tile
                                                              ↓
                                                    Prune (remove low importance)
```

### Importance Dynamics

- **New tiles** start at importance 0.5
- **Each access** adds 0.01 to importance (reinforcement)
- **Decay** subtracts a fixed rate per cycle (clamped at 0)
- **Consolidation** decays originals by 0.2, creates dreams at average importance

### Eviction Policy

When the store is at capacity and a new tile arrives:
1. Find the tile with lowest importance
2. Remove it
3. Insert the new tile

This creates an effective LRU with priority — frequently accessed, important tiles survive; untouched, unimportant tiles get evicted.

### Path Finding

The `MemoryGraph` uses BFS with cumulative strength tracking:

```
strength(path) = Π edges along path
```

The algorithm finds the path with the highest cumulative product of edge strengths between two tiles.

### Consolidation (Dream Phase)

Simple similarity heuristic:
1. Same `TileType` (discriminant match)
2. Shared metadata key-value pair, OR one content string contains the other

When ≥ `min_cluster_size` similar tiles are found:
- A Dream tile is created with summarized content
- Importance = average of cluster members
- All originals are connected to the dream and decayed by `consolidation_decay`

---

## The Math

### Importance Decay

Linear decay with floor:

```
I(t+1) = max(0, I(t) − r)
```

where r is the decay rate. After n unaccessed steps:

```
I(t+n) = max(0, I(0) − n·r)
```

### Reinforcement

```
I(access) = min(1, I + δ)
```

where δ = 0.01 per access. The tension between decay and reinforcement creates a natural attention mechanism: frequently recalled memories persist, neglected ones fade.

### Memory Pressure

```
P = |tiles| / capacity
```

P = 0: store is empty. P = 1: at capacity, next insert triggers eviction. Higher pressure means more aggressive consolidation and pruning are needed.

### Path Strength

For a path through edges e₁, e₂, ..., eₖ with strengths s₁, s₂, ..., sₖ:

```
S(path) = Πᵢ sᵢ
```

This multiplicative model means weak links dramatically reduce path strength, while consistently strong connections produce high-confidence associations.

---

## Test Suite

This crate currently has **0 tests** (the implementation is complete but tests have not been written yet). Contributions welcome.

The API is designed to be testable:
- `MemoryTile::with_id()` and `with_timestamps()` enable deterministic testing
- `MemoryStore::get_untouched()` allows inspection without side effects
- All types implement `Serialize`/`Deserialize` for snapshot testing

---

## License

MIT
