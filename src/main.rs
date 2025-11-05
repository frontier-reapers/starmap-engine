use std::env;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starmap_engine::data::{read_graph_from_file, DataError};
use starmap_engine::graph::graph::StarGraph;
use starmap_engine::graph::pathfinder::shortest_gate_path;
use starmap_engine::spatial::kd_tree::KDTree;
use starmap_engine::sweep::sweep::greedy_sweep_within_radius;
use starmap_engine::System;

static GRAPH: Lazy<StarGraph> = Lazy::new(load_or_sample_graph);

static GRAPH_KD: Lazy<KDTree> = Lazy::new(|| {
    let pts: Vec<[f32; 3]> = GRAPH.systems.iter().map(|s| s.pos).collect();
    KDTree::build(&pts)
});

fn load_or_sample_graph() -> StarGraph {
    match load_graph_from_env() {
        Ok(Some(graph)) => graph,
        Ok(None) => {
            log::info!("STARMAP_DATASET not set; using built-in demo graph");
            sample_graph()
        }
        Err(err) => {
            log::warn!("Failed to load dataset from STARMAP_DATASET: {err}");
            sample_graph()
        }
    }
}

fn load_graph_from_env() -> Result<Option<StarGraph>, DataError> {
    if let Ok(path) = env::var("STARMAP_DATASET") {
        log::info!("Loading dataset from {path}");
        let graph = read_graph_from_file(path)?;
        Ok(Some(graph))
    } else {
        Ok(None)
    }
}

fn sample_graph() -> StarGraph {
    // Tiny demo graph; replace with real starmap index in production.
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
            pos: [0.0, 2.0, 0.0],
        },
    ];
    let adjacency = vec![
        vec![1, 3], // A -> B, D
        vec![0, 2], // B -> A, C
        vec![1],    // C -> B
        vec![0],    // D -> A
    ];
    StarGraph::new(systems, adjacency)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LocationInput {
    Coordinates {
        #[serde(alias = "origin", alias = "center")]
        coords: [f32; 3],
    },
    System {
        system_name: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum EngineRequest {
    Nearest {
        #[serde(flatten)]
        location: LocationInput,
        radius: f32,
        count: usize,
    },
    Path {
        start_id: u32,
        end_id: u32,
    },
    Sweep {
        #[serde(flatten)]
        location: LocationInput,
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
        EngineRequest::Nearest {
            location,
            radius,
            count,
        } => {
            let origin = match resolve_location(&GRAPH, location) {
                Ok(point) => point,
                Err(msg) => return Ok(EngineResponse::Error { message: msg }),
            };
            let kd = &*GRAPH_KD;
            let nn = kd.nearest_n_within_radius(origin, radius, count);
            let systems = nn
                .into_iter()
                .map(|(idx, d)| {
                    let s = &GRAPH.systems[idx];
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
            let g = &*GRAPH;
            let Some(start) = g.index_of_id(start_id) else {
                return Ok(EngineResponse::Error {
                    message: format!("Unknown start_id {}", start_id),
                });
            };
            let Some(goal) = g.index_of_id(end_id) else {
                return Ok(EngineResponse::Error {
                    message: format!("Unknown end_id {}", end_id),
                });
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
                Ok(EngineResponse::Error {
                    message: "No path found".into(),
                })
            }
        }
        EngineRequest::Sweep { location, radius } => {
            let center = match resolve_location(&GRAPH, location) {
                Ok(point) => point,
                Err(msg) => return Ok(EngineResponse::Error { message: msg }),
            };
            let g = &*GRAPH;
            let (indices, total_distance) = greedy_sweep_within_radius(g, center, radius);
            let systems = indices
                .into_iter()
                .map(|idx| {
                    let s = &g.systems[idx];
                    SweepResult {
                        id: s.id,
                        name: s.name.clone(),
                    }
                })
                .collect();
            Ok(EngineResponse::Sweep {
                systems,
                total_distance,
            })
        }
    }
}

fn resolve_location(graph: &StarGraph, location: LocationInput) -> Result<[f32; 3], String> {
    match location {
        LocationInput::Coordinates { coords } => Ok(coords),
        LocationInput::System { system_name } => {
            let Some(index) = graph.index_of_name(&system_name) else {
                return Err(format!("Unknown system_name {system_name}"));
            };
            Ok(graph.systems[index].pos)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let func = service_fn(handler);
    lambda_runtime::run(func).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_from_coordinates() {
        let json = r#"{"kind":"nearest","origin":[1.0,2.0,3.0],"radius":1.0,"count":1}"#;
        let req: EngineRequest = serde_json::from_str(json).expect("parse");
        match req {
            EngineRequest::Nearest { location, .. } => {
                let coords = resolve_location(&sample_graph(), location).expect("coords");
                assert_eq!(coords, [1.0, 2.0, 3.0]);
            }
            _ => panic!("expected nearest"),
        }
    }

    #[test]
    fn location_from_system_name() {
        let json = r#"{"kind":"sweep","system_name":"B","radius":10.0}"#;
        let req: EngineRequest = serde_json::from_str(json).expect("parse");
        match req {
            EngineRequest::Sweep { location, .. } => {
                let coords = resolve_location(&sample_graph(), location).expect("coords");
                assert_eq!(coords, [1.0, 0.0, 0.0]);
            }
            _ => panic!("expected sweep"),
        }
    }

    #[test]
    fn location_unknown_system_errors() {
        let json = r#"{"kind":"nearest","system_name":"Z","radius":5.0,"count":1}"#;
        let req: EngineRequest = serde_json::from_str(json).expect("parse");
        match req {
            EngineRequest::Nearest { location, .. } => {
                let err = resolve_location(&sample_graph(), location).expect_err("missing");
                assert!(err.contains("Unknown system_name"));
            }
            _ => panic!("expected nearest"),
        }
    }
}
