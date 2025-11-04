# starmap-engine

A small Rust engine for spatial queries and pathfinding over a 3D starmap.

Features:

1. **Nearest neighbours**: N nearest systems within a radius using a 3D k‑d tree.
2. **Gate pathfinding**: A* through a gate network using minimal fuel cost (one unit per gate jump).
3. **Sweep optimisation**: Greedy minimum-distance visit of all systems within a radius of a point.

The crate is structured as a reusable library with an AWS Lambda binary entrypoint.

## Layout

- `src/lib.rs` – core `System` type and module wiring.
- `src/spatial/kd_tree.rs` – k‑d tree implementation and nearest‑within‑radius query.
- `src/graph/graph.rs` – starmap graph structure.
- `src/graph/pathfinder.rs` – A* over the gate graph.
- `src/sweep/sweep.rs` – greedy sweep to visit all systems in a radius.
- `src/main.rs` – AWS Lambda handler that exposes three operations:
  - `nearest`
  - `path`
  - `sweep`

## Running tests

```bash
cargo test
```

## Example Lambda event

```json
{
  "kind": "nearest",
  "origin": [0.0, 0.0, 0.0],
  "radius": 3.0,
  "count": 3
}
```

```json
{
  "kind": "path",
  "start_id": 1,
  "end_id": 3
}
```

```json
{
  "kind": "sweep",
  "center": [0.0, 0.0, 0.0],
  "radius": 3.0
}
```

## AWS Lambda

The binary `starmap_lambda` is suitable for deployment to AWS Lambda using the
`provided.al2` runtime and tools such as [`cargo-lambda`](https://www.cargo-lambda.info/).

You will likely want to replace the tiny in-memory sample graph in `src/main.rs`
with your pre-baked index (e.g. from `c3e6.db`) and load it via `include_bytes!`
for best cold-start performance.
