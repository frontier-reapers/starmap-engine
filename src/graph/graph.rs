use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::System;

/// Simple adjacency-list graph over systems.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StarGraph {
    pub systems: Vec<System>,
    /// adjacency[i] lists neighbour indices of systems[i]
    pub adjacency: Vec<Vec<usize>>,
    #[serde(skip)]
    name_index: HashMap<String, usize>,
}

impl StarGraph {
    pub fn new(systems: Vec<System>, adjacency: Vec<Vec<usize>>) -> Self {
        assert_eq!(
            systems.len(),
            adjacency.len(),
            "adjacency must match systems"
        );
        let mut graph = StarGraph {
            systems,
            adjacency,
            name_index: HashMap::new(),
        };
        graph.rebuild_indices();
        graph
    }

    pub fn len(&self) -> usize {
        self.systems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    pub fn index_of_id(&self, id: u32) -> Option<usize> {
        self.systems.iter().position(|s| s.id == id)
    }

    pub fn index_of_name(&self, name: &str) -> Option<usize> {
        self.name_index.get(name).copied()
    }

    pub fn rebuild_indices(&mut self) {
        self.name_index = self
            .systems
            .iter()
            .enumerate()
            .map(|(idx, system)| (system.name.clone(), idx))
            .collect();
    }
}
