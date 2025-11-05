pub mod data;
pub mod graph;
pub mod spatial;
pub mod sweep;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct System {
    pub id: u32,
    pub name: String,
    /// Position in 3D space (e.g. light-years, already transformed)
    pub pos: [f32; 3],
}

impl System {
    pub fn distance(&self, other: &System) -> f32 {
        let dx = self.pos[0] - other.pos[0];
        let dy = self.pos[1] - other.pos[1];
        let dz = self.pos[2] - other.pos[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn distance_to_point(&self, p: [f32; 3]) -> f32 {
        let dx = self.pos[0] - p[0];
        let dy = self.pos[1] - p[1];
        let dz = self.pos[2] - p[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
