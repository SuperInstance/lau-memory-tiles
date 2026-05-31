use lau_memory_tiles::*;

fn make_tile(id: &str, tile_type: TileType, content: &str) -> MemoryTile {
    MemoryTile::new("room1", "agent1", tile_type, content).with_id(id)
}

fn make_tile_with(room_id: &str, agent_id: &str, id: &str, tile_type: TileType, content: &str) -> MemoryTile {
    MemoryTile::new(room_id, agent_id, tile_type, content).with_id(id)
}

// --- TileType tests ---

#[test]
fn tile_type_display() {
    assert_eq!(TileType::Observation.to_string(), "observation");
    assert_eq!(TileType::Action.to_string(), "action");
    assert_eq!(TileType::Thought.to_string(), "thought");
    assert_eq!(TileType::Decision.to_string(), "decision");
    assert_eq!(TileType::Dream.to_string(), "dream");
    assert_eq!(TileType::Error.to_string(), "error");
    assert_eq!(TileType::Signal.to_string(), "signal");
    assert_eq!(TileType::Response.to_string(), "response");
    assert_eq!(TileType::Custom("vibe".to_string()).to_string(), "custom:vibe");
}

#[test]
fn tile_type_is_observable() {
    assert!(TileType::Observation.is_observable());
    assert!(TileType::Action.is_observable());
    assert!(TileType::Signal.is_observable());
    assert!(TileType::Response.is_observable());
    assert!(!TileType::Thought.is_observable());
    assert!(!TileType::Decision.is_observable());
    assert!(!TileType::Dream.is_observable());
}

#[test]
fn tile_type_is_internal() {
    assert!(TileType::Thought.is_internal());
    assert!(TileType::Decision.is_internal());
    assert!(TileType::Dream.is_internal());
    assert!(!TileType::Observation.is_internal());
    assert!(!TileType::Action.is_internal());
}

#[test]
fn tile_type_serde_roundtrip() {
    let types = vec![
        TileType::Observation,
        TileType::Action,
        TileType::Thought,
        TileType::Decision,
        TileType::Dream,
        TileType::Error,
        TileType::Signal,
        TileType::Response,
        TileType::Custom("my_type".into()),
    ];
    for tt in types {
        let json = serde_json::to_string(&tt).unwrap();
        let back: TileType = serde_json::from_str(&json).unwrap();
        assert_eq!(tt, back);
    }
}

#[test]
fn tile_type_equality() {
    assert_eq!(TileType::Observation, TileType::Observation);
    assert_ne!(TileType::Observation, TileType::Action);
    assert_eq!(TileType::Custom("a".into()), TileType::Custom("a".into()));
    assert_ne!(TileType::Custom("a".into()), TileType::Custom("b".into()));
}

// --- MemoryTile tests ---

#[test]
fn tile_new_has_defaults() {
    let tile = MemoryTile::new("room1", "agent1", TileType::Observation, "saw something");
    assert_eq!(tile.room_id, "room1");
    assert_eq!(tile.agent_id, "agent1");
    assert_eq!(tile.content, "saw something");
    assert!((tile.importance - 0.5).abs() < f64::EPSILON);
    assert_eq!(tile.access_count, 0);
    assert!(tile.embeddings.is_empty());
    assert!(tile.connections.is_empty());
    assert!(tile.metadata.is_empty());
    assert!(tile.created_at > 0);
    assert_eq!(tile.created_at, tile.last_accessed);
}

#[test]
fn tile_touch_increments() {
    let mut tile = make_tile("t1", TileType::Action, "did something");
    assert_eq!(tile.access_count, 0);
    tile.touch();
    assert_eq!(tile.access_count, 1);
    tile.touch();
    assert_eq!(tile.access_count, 2);
}

#[test]
fn tile_touch_at_deterministic() {
    let mut tile = make_tile("t1", TileType::Action, "did something");
    tile.touch_at(1000);
    assert_eq!(tile.access_count, 1);
    assert_eq!(tile.last_accessed, 1000);
    tile.touch_at(2000);
    assert_eq!(tile.access_count, 2);
    assert_eq!(tile.last_accessed, 2000);
}

#[test]
fn tile_decay_reduces_importance() {
    let mut tile = make_tile("t1", TileType::Observation, "x").with_importance(0.8);
    tile.decay(0.3);
    assert!((tile.importance - 0.5).abs() < 1e-9);
}

#[test]
fn tile_decay_floor_zero() {
    let mut tile = make_tile("t1", TileType::Observation, "x").with_importance(0.1);
    tile.decay(0.5);
    assert!((tile.importance).abs() < 1e-9);
}

#[test]
fn tile_reinforce_increases_importance() {
    let mut tile = make_tile("t1", TileType::Observation, "x").with_importance(0.3);
    tile.reinforce(0.4);
    assert!((tile.importance - 0.7).abs() < 1e-9);
}

#[test]
fn tile_reinforce_caps_at_one() {
    let mut tile = make_tile("t1", TileType::Observation, "x").with_importance(0.9);
    tile.reinforce(0.5);
    assert!((tile.importance - 1.0).abs() < 1e-9);
}

#[test]
fn tile_connect_to() {
    let mut tile = make_tile("t1", TileType::Observation, "x");
    tile.connect_to("t2");
    assert_eq!(tile.connections, vec!["t2"]);
    // No duplicates
    tile.connect_to("t2");
    assert_eq!(tile.connections, vec!["t2"]);
    tile.connect_to("t3");
    assert_eq!(tile.connections, vec!["t2", "t3"]);
}

#[test]
fn tile_age() {
    let tile = make_tile("t1", TileType::Observation, "x").with_timestamps(1000, 1000);
    assert_eq!(tile.age(3000), 2000);
}

#[test]
fn tile_age_saturating() {
    let tile = make_tile("t1", TileType::Observation, "x").with_timestamps(5000, 5000);
    assert_eq!(tile.age(3000), 0);
}

#[test]
fn tile_is_fresh() {
    let tile = make_tile("t1", TileType::Observation, "x").with_timestamps(1000, 1000);
    assert!(tile.is_fresh(2000, 1001));
    assert!(!tile.is_fresh(2000, 999));
}

#[test]
fn tile_with_importance_clamps() {
    let tile = make_tile("t1", TileType::Observation, "x").with_importance(1.5);
    assert!((tile.importance - 1.0).abs() < 1e-9);
    let tile = make_tile("t1", TileType::Observation, "x").with_importance(-0.5);
    assert!(tile.importance.abs() < 1e-9);
}

#[test]
fn tile_with_metadata() {
    let tile = make_tile("t1", TileType::Observation, "x")
        .with_metadata("source", "sensor")
        .with_metadata("priority", "high");
    assert_eq!(tile.metadata.get("source").unwrap(), "sensor");
    assert_eq!(tile.metadata.get("priority").unwrap(), "high");
}

#[test]
fn tile_with_embeddings() {
    let tile = make_tile("t1", TileType::Observation, "x").with_embeddings(vec![0.1, 0.2, 0.3]);
    assert_eq!(tile.embeddings, vec![0.1, 0.2, 0.3]);
}

#[test]
fn tile_serde_roundtrip() {
    let mut tile = make_tile("t1", TileType::Observation, "saw a thing")
        .with_importance(0.75)
        .with_metadata("key", "val")
        .with_embeddings(vec![0.1, 0.2])
        .with_timestamps(1000, 2000);
    tile.connect_to("t2");
    let json = serde_json::to_string(&tile).unwrap();
    let back: MemoryTile = serde_json::from_str(&json).unwrap();
    assert_eq!(tile.id, back.id);
    assert_eq!(tile.content, back.content);
    assert_eq!(tile.importance, back.importance);
    assert_eq!(tile.embeddings, back.embeddings);
}

// --- MemoryStore tests ---

#[test]
fn store_new() {
    let store = MemoryStore::new(100);
    assert_eq!(store.tile_count(), 0);
    assert_eq!(store.max_tiles, 100);
}

#[test]
fn store_add_and_get() {
    let mut store = MemoryStore::new(100);
    let tile = make_tile("t1", TileType::Observation, "hello");
    store.store(tile);
    assert_eq!(store.tile_count(), 1);
    let got = store.get("t1").unwrap();
    assert_eq!(got.content, "hello");
    // Getting reinforces
    assert_eq!(got.access_count, 2); // store touches + get touches
}

#[test]
fn store_get_nonexistent() {
    let mut store = MemoryStore::new(100);
    assert!(store.get("nope").is_none());
}

#[test]
fn store_eviction() {
    let mut store = MemoryStore::new(3);
    store.store(make_tile("t1", TileType::Observation, "low").with_importance(0.1));
    store.store(make_tile("t2", TileType::Observation, "mid").with_importance(0.5));
    store.store(make_tile("t3", TileType::Observation, "high").with_importance(0.9));

    assert_eq!(store.tile_count(), 3);

    // Adding a 4th should evict t1 (lowest importance)
    store.store(make_tile("t4", TileType::Observation, "new").with_importance(0.7));
    assert_eq!(store.tile_count(), 3);
    assert!(store.get_untouched("t1").is_none());
    assert!(store.get_untouched("t2").is_some());
    assert!(store.get_untouched("t3").is_some());
    assert!(store.get_untouched("t4").is_some());
}

#[test]
fn store_query_by_room() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile_with("r1", "a1", "t1", TileType::Observation, "x"));
    store.store(make_tile_with("r2", "a1", "t2", TileType::Observation, "y"));
    store.store(make_tile_with("r1", "a1", "t3", TileType::Action, "z"));

    let results = store.query(Some("r1"), None, None);
    assert_eq!(results.len(), 2);
}

#[test]
fn store_query_by_type() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x"));
    store.store(make_tile("t2", TileType::Action, "y"));

    let results = store.query(None, Some(&TileType::Observation), None);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t1");
}

#[test]
fn store_query_by_importance() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x").with_importance(0.3));
    store.store(make_tile("t2", TileType::Observation, "y").with_importance(0.8));

    let results = store.query(None, None, Some(0.5));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t2");
}

#[test]
fn store_recent() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "old").with_timestamps(100, 100));
    store.store(make_tile("t2", TileType::Observation, "new").with_timestamps(200, 200));
    store.store(make_tile("t3", TileType::Observation, "mid").with_timestamps(150, 150));

    let recent = store.recent(2);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].id, "t2");
    assert_eq!(recent[1].id, "t3");
}

#[test]
fn store_important() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "low").with_importance(0.2));
    store.store(make_tile("t2", TileType::Observation, "high").with_importance(0.9));
    store.store(make_tile("t3", TileType::Observation, "mid").with_importance(0.5));

    let top = store.important(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].id, "t2");
    assert_eq!(top[1].id, "t3");
}

#[test]
fn store_decay_all() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x").with_importance(0.8));
    store.store(make_tile("t2", TileType::Observation, "y").with_importance(0.5));
    store.decay_all(0.2);

    let t1 = store.get_untouched("t1").unwrap();
    let t2 = store.get_untouched("t2").unwrap();
    assert!((t1.importance - 0.6).abs() < 1e-9);
    assert!((t2.importance - 0.3).abs() < 1e-9);
}

#[test]
fn store_prune() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x").with_importance(0.3));
    store.store(make_tile("t2", TileType::Observation, "y").with_importance(0.7));
    store.store(make_tile("t3", TileType::Observation, "z").with_importance(0.1));

    store.prune(0.2);
    assert_eq!(store.tile_count(), 2);
    assert!(store.get_untouched("t3").is_none());
}

#[test]
fn store_memory_pressure() {
    let mut store = MemoryStore::new(10);
    assert!((store.memory_pressure() - 0.0).abs() < 1e-9);
    store.store(make_tile("t1", TileType::Observation, "x"));
    assert!((store.memory_pressure() - 0.1).abs() < 1e-9);
}

#[test]
fn store_memory_pressure_zero_capacity() {
    let store = MemoryStore::new(0);
    assert!((store.memory_pressure() - 1.0).abs() < 1e-9);
}

#[test]
fn store_remove() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x"));
    let removed = store.remove("t1");
    assert!(removed.is_some());
    assert_eq!(store.tile_count(), 0);
    assert!(store.remove("nonexistent").is_none());
}

#[test]
fn store_overwrite_existing_id() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "old"));
    store.store(make_tile("t1", TileType::Action, "new"));
    assert_eq!(store.tile_count(), 1);
    assert_eq!(store.get_untouched("t1").unwrap().content, "new");
    assert_eq!(store.get_untouched("t1").unwrap().tile_type, TileType::Action);
}

#[test]
fn store_serde_roundtrip() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "hello").with_importance(0.75));
    let json = serde_json::to_string(&store).unwrap();
    let back: MemoryStore = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tile_count(), 1);
    assert_eq!(back.get_untouched("t1").unwrap().content, "hello");
}

// --- MemoryGraph tests ---

#[test]
fn graph_connect() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.8);
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn graph_connect_updates_existing() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.5);
    graph.connect("a", "b", 0.9);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.edges[0].2, 0.9);
}

#[test]
fn graph_disconnect() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.8);
    graph.disconnect("a", "b");
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn graph_connections_of() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.5);
    graph.connect("c", "a", 0.7);

    let conns = graph.connections_of("a");
    assert_eq!(conns.len(), 2);
}

#[test]
fn graph_connections_of_isolated() {
    let graph = MemoryGraph::new();
    assert!(graph.connections_of("x").is_empty());
}

#[test]
fn graph_strongest_path_direct() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.9);
    let path = graph.strongest_path("a", "b").unwrap();
    assert_eq!(path, vec!["a", "b"]);
}

#[test]
fn graph_strongest_path_multi_hop() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.9);
    graph.connect("b", "c", 0.8);
    let path = graph.strongest_path("a", "c").unwrap();
    assert_eq!(path, vec!["a", "b", "c"]);
}

#[test]
fn graph_strongest_path_no_path() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.9);
    assert!(graph.strongest_path("a", "z").is_none());
}

#[test]
fn graph_strongest_path_same_node() {
    let graph = MemoryGraph::new();
    let path = graph.strongest_path("a", "a").unwrap();
    assert_eq!(path, vec!["a"]);
}

#[test]
fn graph_strongest_path_prefers_stronger() {
    let mut graph = MemoryGraph::new();
    // Weak direct path
    graph.connect("a", "c", 0.1);
    // Strong two-hop path
    graph.connect("a", "b", 0.9);
    graph.connect("b", "c", 0.9);

    let path = graph.strongest_path("a", "c").unwrap();
    // Should prefer a->b->c (0.81) over a->c (0.1)
    assert_eq!(path, vec!["a", "b", "c"]);
}

#[test]
fn graph_cluster() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.8);
    graph.connect("b", "c", 0.6);
    graph.connect("a", "c", 0.4);

    let strength = graph.cluster(&["a".to_string(), "b".to_string(), "c".to_string()]);
    // Average: (0.8 + 0.6 + 0.4) / 3
    assert!((strength - 0.6).abs() < 1e-9);
}

#[test]
fn graph_cluster_no_connections() {
    let graph = MemoryGraph::new();
    let strength = graph.cluster(&["a".to_string(), "b".to_string()]);
    assert!((strength).abs() < 1e-9);
}

#[test]
fn graph_cluster_single() {
    let graph = MemoryGraph::new();
    let strength = graph.cluster(&["a".to_string()]);
    assert!((strength).abs() < 1e-9);
}

#[test]
fn graph_default() {
    let graph = MemoryGraph::default();
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn graph_serde_roundtrip() {
    let mut graph = MemoryGraph::new();
    graph.connect("a", "b", 0.5);
    let json = serde_json::to_string(&graph).unwrap();
    let back: MemoryGraph = serde_json::from_str(&json).unwrap();
    assert_eq!(back.edge_count(), 1);
}

// --- MemoryQuery tests ---

#[test]
fn query_no_filters() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x"));
    store.store(make_tile("t2", TileType::Action, "y"));

    let results = MemoryQuery::new().execute(&store);
    assert_eq!(results.len(), 2);
}

#[test]
fn query_by_room_id() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile_with("r1", "a1", "t1", TileType::Observation, "x"));
    store.store(make_tile_with("r2", "a1", "t2", TileType::Observation, "y"));

    let results = MemoryQuery::new().room_id("r1").execute(&store);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t1");
}

#[test]
fn query_by_agent_id() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile_with("r1", "a1", "t1", TileType::Observation, "x"));
    store.store(make_tile_with("r1", "a2", "t2", TileType::Observation, "y"));

    let results = MemoryQuery::new().agent_id("a2").execute(&store);
    assert_eq!(results.len(), 1);
}

#[test]
fn query_by_tile_type() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x"));
    store.store(make_tile("t2", TileType::Action, "y"));

    let results = MemoryQuery::new().tile_type(TileType::Action).execute(&store);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t2");
}

#[test]
fn query_by_importance() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x").with_importance(0.3));
    store.store(make_tile("t2", TileType::Observation, "y").with_importance(0.8));

    let results = MemoryQuery::new().min_importance(0.5).execute(&store);
    assert_eq!(results.len(), 1);
}

#[test]
fn query_by_time_range() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x").with_timestamps(100, 100));
    store.store(make_tile("t2", TileType::Observation, "y").with_timestamps(200, 200));
    store.store(make_tile("t3", TileType::Observation, "z").with_timestamps(300, 300));

    let results = MemoryQuery::new().time_range(150, 250).execute(&store);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t2");
}

#[test]
fn query_by_content() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "hello world"));
    store.store(make_tile("t2", TileType::Observation, "goodbye world"));

    let results = MemoryQuery::new().content_contains("hello").execute(&store);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t1");
}

#[test]
fn query_with_limit() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "x").with_importance(0.9));
    store.store(make_tile("t2", TileType::Observation, "y").with_importance(0.5));
    store.store(make_tile("t3", TileType::Observation, "z").with_importance(0.1));

    let results = MemoryQuery::new().limit(2).execute(&store);
    assert_eq!(results.len(), 2);
    // Sorted by importance
    assert_eq!(results[0].id, "t1");
    assert_eq!(results[1].id, "t2");
}

#[test]
fn query_combined_filters() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile_with("r1", "a1", "t1", TileType::Observation, "hello").with_importance(0.8));
    store.store(make_tile_with("r1", "a1", "t2", TileType::Action, "hello").with_importance(0.8));
    store.store(make_tile_with("r2", "a1", "t3", TileType::Observation, "hello").with_importance(0.8));
    store.store(make_tile_with("r1", "a1", "t4", TileType::Observation, "hello").with_importance(0.2));

    let results = MemoryQuery::new()
        .room_id("r1")
        .tile_type(TileType::Observation)
        .min_importance(0.5)
        .execute(&store);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t1");
}

#[test]
fn query_no_match() {
    let store = MemoryStore::new(100);
    let results = MemoryQuery::new().room_id("nonexistent").execute(&store);
    assert!(results.is_empty());
}

#[test]
fn query_serde_roundtrip() {
    let q = MemoryQuery::new()
        .room_id("r1")
        .min_importance(0.5)
        .content_contains("test")
        .limit(10);
    let json = serde_json::to_string(&q).unwrap();
    let back: MemoryQuery = serde_json::from_str(&json).unwrap();
    assert_eq!(back.room_id, Some("r1".to_string()));
    assert_eq!(back.min_importance, Some(0.5));
    assert_eq!(back.content_contains, Some("test".to_string()));
    assert_eq!(back.limit, Some(10));
}

// --- MemoryConsolidator tests ---

#[test]
fn consolidator_default() {
    let c = MemoryConsolidator::default();
    assert_eq!(c.min_cluster_size, 2);
    assert!((c.consolidation_decay - 0.2).abs() < 1e-9);
}

#[test]
fn consolidator_nothing_to_consolidate() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Observation, "unique1"));
    store.store(make_tile("t2", TileType::Action, "unique2")); // Different type

    let consolidator = MemoryConsolidator::default();
    let created = consolidator.consolidate(&mut store);
    assert!(created.is_empty());
}

#[test]
fn consolidator_merges_similar_tiles() {
    let mut store = MemoryStore::new(100);
    store.store(
        make_tile("t1", TileType::Observation, "saw a bird")
            .with_importance(0.8)
            .with_metadata("category", "animal"),
    );
    store.store(
        make_tile("t2", TileType::Observation, "saw a bird again")
            .with_importance(0.7)
            .with_metadata("category", "animal"),
    );
    store.store(
        make_tile("t3", TileType::Observation, "saw a bird once more")
            .with_importance(0.6)
            .with_metadata("category", "animal"),
    );

    let consolidator = MemoryConsolidator::new(2, 0.2);
    let created = consolidator.consolidate(&mut store);
    assert_eq!(created.len(), 1);

    // Dream tile should exist
    let dream_id = &created[0];
    let dream = store.get_untouched(dream_id).unwrap();
    assert_eq!(dream.tile_type, TileType::Dream);

    // Originals should be decayed
    let t1 = store.get_untouched("t1").unwrap();
    assert!(t1.importance < 0.8);
}

#[test]
fn consolidator_min_cluster_size() {
    let mut store = MemoryStore::new(100);
    store.store(
        make_tile("t1", TileType::Observation, "x")
            .with_metadata("cat", "a"),
    );
    store.store(
        make_tile("t2", TileType::Observation, "y")
            .with_metadata("cat", "a"),
    );

    // Need 3 tiles but only have 2
    let consolidator = MemoryConsolidator::new(3, 0.2);
    let created = consolidator.consolidate(&mut store);
    assert!(created.is_empty());
}

#[test]
fn consolidator_content_similarity() {
    let mut store = MemoryStore::new(100);
    store.store(make_tile("t1", TileType::Error, "connection timeout"));
    store.store(make_tile("t2", TileType::Error, "connection timeout"));

    let consolidator = MemoryConsolidator::new(2, 0.15);
    let created = consolidator.consolidate(&mut store);
    assert_eq!(created.len(), 1);
}

#[test]
fn consolidator_dream_connects_to_members() {
    let mut store = MemoryStore::new(100);
    store.store(
        make_tile("t1", TileType::Observation, "hello")
            .with_metadata("tag", "greeting"),
    );
    store.store(
        make_tile("t2", TileType::Observation, "hello")
            .with_metadata("tag", "greeting"),
    );

    let consolidator = MemoryConsolidator::default();
    let created = consolidator.consolidate(&mut store);
    let dream = store.get_untouched(&created[0]).unwrap();
    assert!(dream.connections.contains(&"t1".to_string()));
    assert!(dream.connections.contains(&"t2".to_string()));
}

#[test]
fn consolidator_serde_roundtrip() {
    let c = MemoryConsolidator::new(3, 0.1);
    let json = serde_json::to_string(&c).unwrap();
    let back: MemoryConsolidator = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_cluster_size, 3);
    assert!((back.consolidation_decay - 0.1).abs() < 1e-9);
}

// --- Integration tests ---

#[test]
fn full_workflow() {
    let mut store = MemoryStore::new(50);

    // Add tiles
    store.store(make_tile("t1", TileType::Observation, "user entered room").with_importance(0.6));
    store.store(make_tile("t2", TileType::Action, "greeted user").with_importance(0.7));
    store.store(make_tile("t3", TileType::Thought, "should I help?").with_importance(0.4));
    store.store(make_tile("t4", TileType::Decision, "offer assistance").with_importance(0.8));

    // Query
    let obs = store.query(None, Some(&TileType::Observation), None);
    assert_eq!(obs.len(), 1);

    // Decay
    store.decay_all(0.1);

    // Important
    let top = store.important(2);
    assert_eq!(top.len(), 2);

    // Connect tiles
    let mut graph = MemoryGraph::new();
    graph.connect("t1", "t2", 0.9);
    graph.connect("t2", "t4", 0.8);
    let path = graph.strongest_path("t1", "t4").unwrap();
    assert_eq!(path.len(), 3);

    // Prune
    store.prune(0.2);
    assert!(store.tile_count() > 0);
}

#[test]
fn memory_lifecycle() {
    let mut store = MemoryStore::new(10);

    // Fill up
    for i in 0..10 {
        store.store(
            make_tile(&format!("t{i}"), TileType::Observation, &format!("event {i}"))
                .with_importance(0.1 * (i as f64 + 1.0).min(1.0)),
        );
    }
    assert_eq!(store.tile_count(), 10);

    // Add one more -> eviction
    store.store(make_tile("t_new", TileType::Action, "overflow").with_importance(0.5));
    assert_eq!(store.tile_count(), 10);

    // Consolidate
    let consolidator = MemoryConsolidator::new(2, 0.1);
    let _ = consolidator.consolidate(&mut store);

    // Memory pressure
    let pressure = store.memory_pressure();
    assert!(pressure > 0.0 && pressure <= 1.0);
}
