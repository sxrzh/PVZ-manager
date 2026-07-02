use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_json::from_str;
use super::models::ManagerData;

pub const USERDATA_PATH: &str = "C:\\ProgramData\\PopCap Games\\PlantsVsZombies\\userdata";

pub fn get_save_path() -> PathBuf {
    // To change the path for saving data of PVZ-manager, change this function.
    // Currently it is C:\Users\*\AppData\Local\PVZ-Manager
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("PVZ-manager")
}

pub fn init_save_path() -> Result<()> {
    let path = get_save_path();
    if !path.exists() {
        fs::create_dir_all(&path).context("Failed to create save directory")?;
    }
    Ok(())
}

pub fn load_manager_data() -> Result<ManagerData> {
    let path = get_save_path().join("manager.json");
    if !path.exists() {
        return Ok(ManagerData::new());
    }
    let content = fs::read_to_string(&path).context("Failed to read manager.json")?;
    from_str(&content).context("Failed to parse manager.json")
}

pub fn save_manager_data(data: &ManagerData) -> Result<()> {
    let path = get_save_path().join("manager.json");
    let content = serde_json::to_string_pretty(data).context("Failed to serialize manager data")?;
    let mut file = File::create(&path).context("Failed to create manager.json")?;
    file.write_all(content.as_bytes()).context("Failed to write manager.json")?;
    Ok(())
}

pub fn get_backup_file_path(user_id: u32, game_id: u32, node_id: u32) -> PathBuf {
    get_save_path().join(format!("game{}_{}backup{}.dat", user_id, game_id, node_id))
}

pub fn get_game_file_path(user_id: u32, game_id: u32) -> PathBuf {
    Path::new(USERDATA_PATH).join(format!("game{}_{}.dat", user_id, game_id))
}

pub fn backup_game_file(user_id: u32, game_id: u32, node_id: u32) -> Result<bool> {
    let source = get_game_file_path(user_id, game_id);
    if !source.exists() {
        return Ok(false);
    }
    let dest = get_backup_file_path(user_id, game_id, node_id);
    fs::copy(&source, &dest).context("Failed to copy game file")?;
    Ok(true)
}

pub fn restore_game_file(user_id: u32, game_id: u32, node_id: u32) -> Result<bool> {
    let source = get_backup_file_path(user_id, game_id, node_id);
    if !source.exists() {
        return Ok(false);
    }
    let dest = get_game_file_path(user_id, game_id);
    fs::copy(&source, &dest).context("Failed to copy backup file")?;
    Ok(true)
}

pub fn delete_game_file(user_id: u32, game_id: u32) -> Result<bool> {
    let dest = get_game_file_path(user_id, game_id);
    if !dest.exists() {
        return Ok(false);
    }
    fs::remove_file(&dest).context("Failed to remove userdata file")?;
    Ok(true)
}

pub fn delete_backup_file(user_id: u32, game_id: u32, node_id: u32) -> Result<bool> {
    let path = get_backup_file_path(user_id, game_id, node_id);
    if !path.exists() {
        return Ok(false);
    }
    fs::remove_file(&path).context("Failed to delete backup file")?;
    Ok(true)
}

pub fn check_game_file_exists(user_id: u32, game_id: u32) -> bool {
    get_game_file_path(user_id, game_id).exists()
}

pub fn load_game_ids() -> Result<std::collections::HashMap<String, u32>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("game_id.json");
    let content = fs::read_to_string(&path).context("Failed to read game_id.json")?;
    from_str(&content).context("Failed to parse game_id.json")
}

pub fn search_game_id(game_ids: &std::collections::HashMap<String, u32>, query: &str) -> Vec<(String, u32)> {
    let mut results = Vec::new();
    for (name, id) in game_ids {
        if name.contains(query) || id.to_string().contains(query) {
            results.push((name.clone(), *id));
        }
    }
    results.sort_by_key(|(_, id)| *id);
    results
}
