use std::fs;
use std::io::Cursor;
use std::path::Path;

use bincode::ErrorKind;
use thiserror::Error;

use crate::graph::graph::StarGraph;

/// Compression level used when encoding serialized graph data.
///
/// We use a named constant to make the chosen level explicit because the
/// dataset bundles benefit from aggressive compression to reduce deployment
/// size.
const GRAPH_COMPRESSION_LEVEL: i32 = 19;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialize(#[from] Box<ErrorKind>),
    #[error("Compression error: {0}")]
    Compression(#[source] std::io::Error),
}

pub fn serialize_graph(graph: &StarGraph) -> Result<Vec<u8>, DataError> {
    let encoded = bincode::serialize(graph)?;
    let mut cursor = Cursor::new(encoded);
    zstd::stream::encode_all(&mut cursor, GRAPH_COMPRESSION_LEVEL).map_err(DataError::Compression)
}

pub fn deserialize_graph(bytes: &[u8]) -> Result<StarGraph, DataError> {
    let mut cursor = Cursor::new(bytes);
    let decoded = zstd::stream::decode_all(&mut cursor).map_err(DataError::Compression)?;
    let mut graph: StarGraph = bincode::deserialize(&decoded)?;
    graph.rebuild_indices();
    Ok(graph)
}

pub fn write_graph_to_file<P: AsRef<Path>>(graph: &StarGraph, path: P) -> Result<(), DataError> {
    let bytes = serialize_graph(graph)?;
    fs::write(path, bytes)?;
    Ok(())
}

pub fn read_graph_from_file<P: AsRef<Path>>(path: P) -> Result<StarGraph, DataError> {
    let bytes = fs::read(path)?;
    deserialize_graph(&bytes)
}
