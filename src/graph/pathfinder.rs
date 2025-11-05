use crate::graph::graph::StarGraph;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Clone, Debug)]
pub struct PathStep {
    pub system_index: usize,
    pub cost: f32,
}

/// A* pathfinding over the gate graph, using 3D Euclidean distance as a heuristic.
/// Cost model: each gate jump has cost 1.0 (minimal fuel usage).
pub fn shortest_gate_path(graph: &StarGraph, start: usize, goal: usize) -> Option<Vec<PathStep>> {
    if start == goal {
        return Some(vec![PathStep {
            system_index: start,
            cost: 0.0,
        }]);
    }

    #[derive(Copy, Clone, Debug)]
    struct Node {
        idx: usize,
        f_score: f32,
    }

    impl Eq for Node {}

    impl PartialEq for Node {
        fn eq(&self, other: &Self) -> bool {
            self.f_score.eq(&other.f_score)
        }
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .f_score
                .partial_cmp(&self.f_score)
                .unwrap_or(Ordering::Equal)
        }
    }

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut open = BinaryHeap::new();
    open.push(Node {
        idx: start,
        f_score: heuristic(graph, start, goal),
    });

    let mut came_from: HashMap<usize, usize> = HashMap::new();
    let mut g_score: HashMap<usize, f32> = HashMap::new();
    g_score.insert(start, 0.0);

    while let Some(Node { idx: current, .. }) = open.pop() {
        if current == goal {
            return Some(reconstruct_path(&came_from, current));
        }

        let current_g = *g_score.get(&current).unwrap_or(&f32::INFINITY);

        for &neighbor in &graph.adjacency[current] {
            let tentative_g = current_g + 1.0; // one gate jump
            if tentative_g < *g_score.get(&neighbor).unwrap_or(&f32::INFINITY) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                let f = tentative_g + heuristic(graph, neighbor, goal);
                open.push(Node {
                    idx: neighbor,
                    f_score: f,
                });
            }
        }
    }

    None
}

fn heuristic(graph: &StarGraph, from: usize, to: usize) -> f32 {
    let a = &graph.systems[from];
    let b = &graph.systems[to];
    a.distance(b)
}

fn reconstruct_path(came_from: &HashMap<usize, usize>, mut current: usize) -> Vec<PathStep> {
    let mut total_path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        total_path.push(current);
    }
    total_path.reverse();

    let mut result = Vec::with_capacity(total_path.len());
    let mut cost = 0.0_f32;
    for (i, idx) in total_path.iter().enumerate() {
        if i > 0 {
            cost += 1.0;
        }
        result.push(PathStep {
            system_index: *idx,
            cost,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::System;

    #[test]
    fn simple_triangle_path() {
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
        ];
        let adjacency = vec![
            vec![1],    // A -> B
            vec![0, 2], // B -> A, C
            vec![1],    // C -> B
        ];
        let graph = StarGraph::new(systems, adjacency);
        let path = shortest_gate_path(&graph, 0, 2).expect("path");
        let ids: Vec<u32> = path
            .iter()
            .map(|p| graph.systems[p.system_index].id)
            .collect();
        assert_eq!(ids, vec![1, 2, 3]);
        assert!((path.last().unwrap().cost - 2.0).abs() < 1e-5);
    }
}
