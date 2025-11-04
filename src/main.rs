use lambda_runtime::{service_fn, Error, LambdaEvent};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starmap_engine::graph::graph::StarGraph;
use starmap_engine::graph::pathfinder::shortest_gate_path;
use starmap_engine::spatial::kd_tree::KDTree;
use starmap_engine::sweep::sweep::greedy_sweep_within_radius;
use starmap_engine::System;

static SAMPLE_GRAPH: Lazy<StarGraph> = Lazy::new(|| {
    // Tiny demo graph; replace with real starmap index in production.
    let systems = vec![
        System { id: 1, name: "A".into(), pos: [0.0, 0.0, 0.0] },
        System { id: 2, name: "B".into(), pos: [1.0, 0.0, 0.0] },
        System { id: 3, name: "C".into(), pos: [2.0, 0.0, 0.0] },
        System { id: 4, name: "D".into(), pos: [0.0, 2.0, 0.0] },
    ];
    let adjacency = vec![
        vec![1, 3],    // A -> B, D
        vec![0, 2],    // B -> A, C
        vec![1],       // C -> B
        vec![0],       // D -> A
    ];
    StarGraph::new(systems, adjacency)
});

static SAMPLE_KD: Lazy<KDTree> = Lazy::new(|| {
    let pts: Vec<[f32; 3]> = SAMPLE_GRAPH.systems.iter().map(|s| s.pos).collect();
    KDTree::build(&pts)
});

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum EngineRequest {
    Nearest {
        origin: [f32; 3],
        radius: f32,
        count: usize,
    },
    Path {
        start_id: u32,
        end_id: u32,
    },
    Sweep {
        center: [f32; 3],
        radius: f32,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum EngineResponse {
    Nearest {
        systems: Vec<NearestResult>,
    },
    Path {
        systems: Vec<PathResult>,
    },
    Sweep {
        systems: Vec<SweepResult>,
        total_distance: f32,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Serialize)]
struct NearestResult {
    id: u32,
    name: String,
    distance: f32,
}

#[derive(Debug, Serialize)]
struct PathResult {
    id: u32,
    name: String,
    cumulative_cost: f32,
}

#[derive(Debug, Serialize)]
struct SweepResult {
    id: u32,
    name: String,
}

async fn handler(event: LambdaEvent<EngineRequest>) -> Result<EngineResponse, Error> {
    let req = event.payload;
    match req {
        EngineRequest::Nearest { origin, radius, count } => {
            let kd = &*SAMPLE_KD;
            let nn = kd.nearest_n_within_radius(origin, radius, count);
            let systems = nn
                .into_iter()
                .map(|(idx, d)| {
                    let s = &SAMPLE_GRAPH.systems[idx];
                    NearestResult {
                        id: s.id,
                        name: s.name.clone(),
                        distance: d,
                    }
                })
                .collect();
            Ok(EngineResponse::Nearest { systems })
        }
        EngineRequest::Path { start_id, end_id } => {
            let g = &*SAMPLE_GRAPH;
            let Some(start) = g.index_of_id(start_id) else {
                return Ok(EngineResponse::Error { message: format!("Unknown start_id {}", start_id) });
            };
            let Some(goal) = g.index_of_id(end_id) else {
                return Ok(EngineResponse::Error { message: format!("Unknown end_id {}", end_id) });
            };
            if let Some(path) = shortest_gate_path(g, start, goal) {
                let systems = path
                    .into_iter()
                    .map(|step| {
                        let s = &g.systems[step.system_index];
                        PathResult {
                            id: s.id,
                            name: s.name.clone(),
                            cumulative_cost: step.cost,
                        }
                    })
                    .collect();
                Ok(EngineResponse::Path { systems })
            } else {
                Ok(EngineResponse::Error { message: "No path found".into() })
            }
        }
        EngineRequest::Sweep { center, radius } => {
            let g = &*SAMPLE_GRAPH;
            let (indices, total_distance) = greedy_sweep_within_radius(g, center, radius);
            let systems = indices
                .into_iter()
                .map(|idx| {
                    let s = &g.systems[idx];
                    SweepResult { id: s.id, name: s.name.clone() }
                })
                .collect();
            Ok(EngineResponse::Sweep { systems, total_distance })
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let func = service_fn(handler);
    lambda_runtime::run(func).await
}
