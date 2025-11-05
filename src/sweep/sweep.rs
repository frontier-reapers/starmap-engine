use crate::graph::graph::StarGraph;

/// Greedy sweep: starting from the closest node to `center` within `radius`,
/// repeatedly visit the nearest unvisited node within that radius.
///
/// Returns (ordered_indices, total_distance).
pub fn greedy_sweep_within_radius(
    graph: &StarGraph,
    center: [f32; 3],
    radius: f32,
) -> (Vec<usize>, f32) {
    let mut candidates: Vec<usize> = graph
        .systems
        .iter()
        .enumerate()
        .filter_map(|(idx, s)| {
            let dx = s.pos[0] - center[0];
            let dy = s.pos[1] - center[1];
            let dz = s.pos[2] - center[2];
            let dist2 = dx * dx + dy * dy + dz * dz;
            if dist2 <= radius * radius {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        return (Vec::new(), 0.0);
    }

    // Start at candidate closest to center
    candidates.sort_by(|&a, &b| {
        let da = graph.systems[a].distance_to_point(center);
        let db = graph.systems[b].distance_to_point(center);
        da.partial_cmp(&db).unwrap()
    });

    let mut path = Vec::new();
    let mut total_distance = 0.0_f32;

    let mut current = candidates.remove(0);
    path.push(current);

    while !candidates.is_empty() {
        let (next_idx, next_pos) = candidates
            .iter()
            .enumerate()
            .map(|(i, &idx)| (i, graph.systems[idx].distance(&graph.systems[current])))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap();

        total_distance += next_pos;
        current = candidates.remove(next_idx);
        path.push(current);
    }

    (path, total_distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::System;

    #[test]
    fn sweep_visits_all_within_radius() {
        let systems = vec![
            System {
                id: 1,
                name: "A".into(),
                pos: [0.0, 0.0, 0.0],
            },
            System {
                id: 2,
                name: "B".into(),
                pos: [1.0, 0.0, 0.0],
            },
            System {
                id: 3,
                name: "C".into(),
                pos: [2.0, 0.0, 0.0],
            },
            System {
                id: 4,
                name: "D".into(),
                pos: [10.0, 0.0, 0.0],
            },
        ];
        let adjacency = vec![vec![], vec![], vec![], vec![]];
        let graph = StarGraph::new(systems, adjacency);

        let (path, dist) = greedy_sweep_within_radius(&graph, [0.0, 0.0, 0.0], 3.0);
        let ids: Vec<u32> = path.iter().map(|&i| graph.systems[i].id).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&1) && ids.contains(&2) && ids.contains(&3));
        assert!(dist > 0.0);
    }
}
