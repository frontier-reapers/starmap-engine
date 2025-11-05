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
  "system_name": "A",
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

Both the `nearest` and `sweep` requests accept either explicit coordinates via
`origin`/`center` fields **or** a `system_name` that is resolved against the
loaded starmap dataset.

## Dataset pipeline

Run the dataset builder to download the latest
[`evefrontier_datasets`](https://github.com/Scetrov/evefrontier_datasets)
release and emit a compressed bundle suitable for Lambda deployment:

```bash
cargo run --bin build_dataset --features dataset-builder
```

The dedicated `dataset-builder` feature keeps the Lambda runtime lean by
compiling the dataset tooling and its dependencies only when explicitly
requested.

The command stores the resulting files in `data/`:

- `starmap.bin` – Zstandard-compressed `StarGraph` ready for inclusion in the Lambda package.
- `starmap.meta.json` – Build metadata (release tag, counts, timestamp).

## AWS Lambda

The binary `starmap_lambda` is suitable for deployment to AWS Lambda using the
`provided.al2` runtime and tools such as [`cargo-lambda`](https://www.cargo-lambda.info/).

Set the `STARMAP_DATASET` environment variable to the path of the compressed
dataset (for example, `data/starmap.bin`) to have the Lambda load it at startup.
If the variable is unset or loading fails, the handler falls back to a small
in-memory demo graph.
