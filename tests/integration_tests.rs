use starmap_engine::graph::graph::StarGraph;
use starmap_engine::graph::pathfinder::shortest_gate_path;
use starmap_engine::spatial::kd_tree::KDTree;
use starmap_engine::sweep::sweep::greedy_sweep_within_radius;
use starmap_engine::System;

#[test]
fn integration_end_to_end_small_graph() {
    let systems = vec![
        System { id: 1, name: "A".into(), pos: [0.0, 0.0, 0.0] },
        System { id: 2, name: "B".into(), pos: [1.0, 0.0, 0.0] },
        System { id: 3, name: "C".into(), pos: [2.0, 0.0, 0.0] },
    ];
    let adjacency = vec![vec![1], vec![0, 2], vec![1]];
    let graph = StarGraph::new(systems.clone(), adjacency);

    // KD-tree nearest
    let pts: Vec<[f32; 3]> = systems.iter().map(|s| s.pos).collect();
    let kd = KDTree::build(&pts);
    let nn = kd.nearest_n_within_radius([0.0, 0.0, 0.0], 2.0, 3);
    assert!(!nn.is_empty());

    // Pathfinding
    let path = shortest_gate_path(&graph, 0, 2).expect("path");
    assert_eq!(path.len(), 3);

    // Sweep
    let (sweep, total) = greedy_sweep_within_radius(&graph, [0.0, 0.0, 0.0], 5.0);
    assert_eq!(sweep.len(), 3);
    assert!(total > 0.0);
}
