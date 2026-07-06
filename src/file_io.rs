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

pub fn load_game_ids() -> std::collections::HashMap<String, u32> {
    let mut map = std::collections::HashMap::new();
    map.insert("冒险模式".to_string(), 0);
    map.insert("生存模式：白天".to_string(), 1);
    map.insert("生存模式：黑夜".to_string(), 2);
    map.insert("生存模式：泳池".to_string(), 3);
    map.insert("生存模式：浓雾".to_string(), 4);
    map.insert("生存模式：屋顶".to_string(), 5);
    map.insert("生存模式：白天（困难）".to_string(), 6);
    map.insert("生存模式：黑夜（困难）".to_string(), 7);
    map.insert("生存模式：泳池（困难）".to_string(), 8);
    map.insert("生存模式：浓雾（困难）".to_string(), 9);
    map.insert("生存模式：屋顶（困难）".to_string(), 10);
    map.insert("生存模式：泳池（无尽）".to_string(), 13);
    map.insert("植物僵尸".to_string(), 16);
    map.insert("坚果保龄球模式".to_string(), 17);
    map.insert("老虎机".to_string(), 18);
    map.insert("雨中种植物".to_string(), 19);
    map.insert("宝石迷阵".to_string(), 20);
    map.insert("隐形食脑者".to_string(), 21);
    map.insert("看星星".to_string(), 22);
    map.insert("僵尸水族馆".to_string(), 23);
    map.insert("宝石迷阵转转看".to_string(), 24);
    map.insert("小僵尸大麻烦".to_string(), 25);
    map.insert("保护传送门".to_string(), 26);
    map.insert("你看，他们像柱子一样".to_string(), 27);
    map.insert("雪橇区".to_string(), 28);
    map.insert("僵尸快跑".to_string(), 29);
    map.insert("锤僵尸".to_string(), 30);
    map.insert("谁笑到最后".to_string(), 31);
    map.insert("植物僵尸2".to_string(), 32);
    map.insert("坚果保龄球2".to_string(), 33);
    map.insert("跳跳舞会".to_string(), 34);
    map.insert("僵王博士的复仇".to_string(), 35);
    map.insert("破罐者".to_string(), 51);
    map.insert("全部留下".to_string(), 52);
    map.insert("第3个罐子".to_string(), 53);
    map.insert("连锁反应".to_string(), 54);
    map.insert("M的意思是金属".to_string(), 55);
    map.insert("胆怯的制陶工".to_string(), 56);
    map.insert("变戏法".to_string(), 57);
    map.insert("另一个连锁反应".to_string(), 58);
    map.insert("罐子王牌".to_string(), 59);
    map.insert("无尽的试炼".to_string(), 60);
    map.insert("我是僵尸！".to_string(), 61);
    map.insert("我也是僵尸！".to_string(), 62);
    map.insert("你能铲了它么？".to_string(), 63);
    map.insert("完全傻了".to_string(), 64);
    map.insert("死亡飞艇".to_string(), 65);
    map.insert("我烂了！".to_string(), 66);
    map.insert("将是摇摆".to_string(), 67);
    map.insert("三连击".to_string(), 68);
    map.insert("你所有大脑都是属于我的".to_string(), 69);
    map.insert("我是僵尸无尽版".to_string(), 70);
    map
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
