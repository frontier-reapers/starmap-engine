use crate::System;

/// Simple adjacency-list graph over systems.
#[derive(Clone, Debug)]
pub struct StarGraph {
    pub systems: Vec<System>,
    /// adjacency[i] lists neighbour indices of systems[i]
    pub adjacency: Vec<Vec<usize>>,
}

impl StarGraph {
    pub fn new(systems: Vec<System>, adjacency: Vec<Vec<usize>>) -> Self {
        assert_eq!(systems.len(), adjacency.len(), "adjacency must match systems");
        StarGraph { systems, adjacency }
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
}
