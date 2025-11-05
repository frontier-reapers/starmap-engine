use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use log::{info, warn};
use reqwest::blocking::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use starmap_engine::data::write_graph_to_file;
use starmap_engine::graph::graph::StarGraph;
use starmap_engine::System;
use tempfile::NamedTempFile;

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Serialize)]
struct DatasetMetadata {
    release_tag: String,
    asset_name: String,
    asset_url: String,
    systems: usize,
    directed_edges: usize,
    generated_at_epoch: u64,
}

fn main() -> Result<()> {
    env_logger::init();

    let client = Client::builder()
        .user_agent("starmap-engine-dataset-builder/0.1")
        .build()?;

    let release = fetch_latest_release(&client)?;
    let asset = select_sqlite_asset(&release)?;

    info!(
        "Downloading dataset asset {} from {}",
        asset.name, asset.browser_download_url
    );

    let temp_file = download_asset(&client, &asset.browser_download_url)?;
    let (graph, edge_count) = build_graph_from_sqlite(temp_file.path())
        .with_context(|| "failed to build graph from SQLite dataset")?;

    let output_dir = PathBuf::from("data");
    fs::create_dir_all(&output_dir).context("failed to create data output directory")?;
    let dataset_path = output_dir.join("starmap.bin");
    write_graph_to_file(&graph, &dataset_path)
        .with_context(|| format!("failed to write dataset to {}", dataset_path.display()))?;

    let metadata = DatasetMetadata {
        release_tag: release.tag_name.clone(),
        asset_name: asset.name.clone(),
        asset_url: asset.browser_download_url.clone(),
        systems: graph.len(),
        directed_edges: edge_count,
        generated_at_epoch: current_epoch_seconds(),
    };

    let metadata_path = output_dir.join("starmap.meta.json");
    let metadata_json = serde_json::to_vec_pretty(&metadata)?;
    fs::write(&metadata_path, metadata_json)
        .with_context(|| format!("failed to write metadata to {}", metadata_path.display()))?;

    info!(
        "Wrote compact dataset to {} ({} systems, {} edges)",
        dataset_path.display(),
        metadata.systems,
        metadata.directed_edges
    );

    Ok(())
}

fn fetch_latest_release(client: &Client) -> Result<Release> {
    let url = "https://api.github.com/repos/Scetrov/evefrontier_datasets/releases/latest";
    let response = client
        .get(url)
        .send()
        .with_context(|| format!("failed to query {url}"))?
        .error_for_status()
        .context("GitHub API returned an error status")?;
    let release: Release = response.json().context("failed to parse release payload")?;
    Ok(release)
}

fn select_sqlite_asset(release: &Release) -> Result<&Asset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(".db"))
        .ok_or_else(|| anyhow!("latest release does not contain a SQLite asset"))
}

fn download_asset(client: &Client, url: &str) -> Result<NamedTempFile> {
    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("failed to download {url}"))?
        .error_for_status()
        .context("dataset download returned an error status")?;
    let mut file = NamedTempFile::new()?;
    response.copy_to(&mut file)?;
    Ok(file)
}

fn build_graph_from_sqlite(path: &Path) -> Result<(StarGraph, usize)> {
    let conn = Connection::open(path)
        .with_context(|| format!("failed to open SQLite database at {}", path.display()))?;

    let mut systems = Vec::new();
    let mut id_to_index = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT solarSystemId, name, centerX, centerY, centerZ FROM SolarSystems ORDER BY solarSystemId",
        )?;
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let x: f64 = row.get(2)?;
            let y: f64 = row.get(3)?;
            let z: f64 = row.get(4)?;
            Ok((id as u32, name, [x as f32, y as f32, z as f32]))
        })?;
        for (idx, row) in rows.enumerate() {
            let (id, name, pos) = row?;
            id_to_index.insert(id, idx);
            systems.push(System { id, name, pos });
        }
    }

    let mut adjacency = vec![Vec::new(); systems.len()];
    {
        let mut stmt = conn.prepare("SELECT fromSystemId, toSystemId FROM Jumps")?;
        let rows = stmt.query_map([], |row| {
            let from: i64 = row.get(0)?;
            let to: i64 = row.get(1)?;
            Ok((from as u32, to as u32))
        })?;
        for row in rows {
            let (from, to) = row?;
            let Some(&from_idx) = id_to_index.get(&from) else {
                warn!("Jumps entry references missing fromSystemId {from}");
                continue;
            };
            let Some(&to_idx) = id_to_index.get(&to) else {
                warn!("Jumps entry references missing toSystemId {to}");
                continue;
            };
            adjacency[from_idx].push(to_idx);
        }
    }

    let mut edge_count = 0usize;
    for neighbours in &mut adjacency {
        neighbours.sort_unstable();
        neighbours.dedup();
        edge_count += neighbours.len();
    }

    let graph = StarGraph::new(systems, adjacency);
    Ok((graph, edge_count))
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
