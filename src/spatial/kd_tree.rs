use serde::{Deserialize, Serialize};

/// Node in a 3D k-d tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KDNode {
    pub point: [f32; 3],
    pub index: usize,
    pub axis: usize,
    pub left: Option<Box<KDNode>>,
    pub right: Option<Box<KDNode>>,
}

/// Simple 3D k-d tree supporting N-nearest-within-radius queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KDTree {
    pub root: Option<Box<KDNode>>,
}

impl KDTree {
    pub fn build(points: &[[f32; 3]]) -> Self {
        let mut indices: Vec<usize> = (0..points.len()).collect();
        let root = Self::build_recursive(points, &mut indices, 0);
        KDTree { root }
    }

    fn build_recursive(
        points: &[[f32; 3]],
        idx: &mut [usize],
        depth: usize,
    ) -> Option<Box<KDNode>> {
        use core::cmp::Ordering;

        if idx.is_empty() {
            return None;
        }

        let axis = depth % 3;
        idx.sort_by(|&a, &b| {
            points[a][axis]
                .partial_cmp(&points[b][axis])
                .unwrap_or(Ordering::Equal)
        });
        let mid = idx.len() / 2;
        let median = idx[mid];

        Some(Box::new(KDNode {
            point: points[median],
            index: median,
            axis,
            left: Self::build_recursive(points, &mut idx[..mid], depth + 1),
            right: Self::build_recursive(points, &mut idx[mid + 1..], depth + 1),
        }))
    }

    /// Returns up to `n` nearest neighbours within the given radius of the target point.
    pub fn nearest_n_within_radius(
        &self,
        target: [f32; 3],
        radius: f32,
        n: usize,
    ) -> Vec<(usize, f32)> {
        let mut results = Vec::new();
        let radius2 = radius * radius;
        self.search_recursive(&self.root, target, radius2, &mut results);
        // sort ascending by distance
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(n);
        results
    }

    #[allow(clippy::only_used_in_recursion)]
    fn search_recursive(
        &self,
        node: &Option<Box<KDNode>>,
        target: [f32; 3],
        radius2: f32,
        results: &mut Vec<(usize, f32)>,
    ) {
        if let Some(noderef) = node {
            let dx = noderef.point[0] - target[0];
            let dy = noderef.point[1] - target[1];
            let dz = noderef.point[2] - target[2];
            let dist2 = dx * dx + dy * dy + dz * dz;
            if dist2 <= radius2 {
                results.push((noderef.index, dist2.sqrt()));
            }

            let axis = noderef.axis;
            let delta = target[axis] - noderef.point[axis];
            let (first, second) = if delta < 0.0 {
                (&noderef.left, &noderef.right)
            } else {
                (&noderef.right, &noderef.left)
            };

            self.search_recursive(first, target, radius2, results);
            if delta * delta <= radius2 {
                self.search_recursive(second, target, radius2, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KDTree;

    #[test]
    fn nearest_n_within_radius_basic() {
        let pts = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let kd = KDTree::build(&pts);
        let res = kd.nearest_n_within_radius([0.0, 0.0, 0.0], 1.5, 3);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].0, 0);
        assert_eq!(res[1].0, 1);
    }
}
