use crate::types::{MeshEntry, MESHES_JSON};
use anyhow::{Context, Result};
use std::path::Path;

/// Load meshes.json from disk. Returns empty vec if file does not exist.
pub fn load_meshes() -> Result<Vec<MeshEntry>> {
    let path = Path::new(MESHES_JSON);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {MESHES_JSON}"))?;
    let meshes: Vec<MeshEntry> =
        serde_json::from_str(&content).with_context(|| format!("Failed to parse {MESHES_JSON}"))?;
    Ok(meshes)
}

/// Save meshes to meshes.json, overwriting existing file.
pub fn save_meshes(meshes: &[MeshEntry]) -> Result<()> {
    let json = serde_json::to_string_pretty(meshes)
        .context("Failed to serialize meshes")?;
    std::fs::write(MESHES_JSON, json)
        .with_context(|| format!("Failed to write {MESHES_JSON}"))?;
    Ok(())
}

/// Upsert a mesh entry by address (add if new, update if existing).
pub fn upsert_mesh(meshes: &mut Vec<MeshEntry>, entry: MeshEntry) {
    let addr = entry.address.to_lowercase();
    if let Some(existing) = meshes.iter_mut().find(|m| m.address.to_lowercase() == addr) {
        *existing = entry;
    } else {
        meshes.push(entry);
    }
}
