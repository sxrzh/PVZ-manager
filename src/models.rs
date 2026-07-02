use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: u32,
    pub name: String,
    pub parent_id: i32,
    pub created_at: DateTime<Local>,
    pub note: String,
    pub file_deleted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameData {
    pub nodes: Vec<Node>,
    pub cur: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManagerData {
    pub user_id: Option<u32>,
    pub data: std::collections::HashMap<u32, GameData>,
}

impl ManagerData {
    pub fn new() -> Self {
        Self {
            user_id: None,
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_create_game_data(&mut self, game_id: u32) -> &mut GameData {
        self.data.entry(game_id).or_insert_with(|| {
            let mut nodes = Vec::new();
            nodes.push(Node {
                id: 0,
                name: "根节点".to_string(),
                parent_id: -1,
                created_at: chrono::Local::now(),
                note: "根节点，表示没有存档".to_string(),
                file_deleted: false,
            });
            GameData { nodes, cur: 0 }
        })
    }

    /// 根节点编号始终是 0，此方法确保 GameData 已初始化
    pub fn get_root_id(&mut self, game_id: u32) -> u32 {
        self.get_or_create_game_data(game_id);
        0
    }

    pub fn create_node(&mut self, game_id: u32, parent_id: i32, name: String, note: String) -> u32 {
        let game_data = self.get_or_create_game_data(game_id);
        let node_id = game_data.nodes.len() as u32;
        game_data.nodes.push(Node {
            id: node_id,
            name,
            parent_id,
            created_at: chrono::Local::now(),
            note,
            file_deleted: false,
        });
        node_id
    }

    pub fn find_node(&self, game_id: u32, node_id: u32) -> Option<&Node> {
        self.data.get(&game_id)?.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn find_node_mut(&mut self, game_id: u32, node_id: u32) -> Option<&mut Node> {
        self.data.get_mut(&game_id)?.nodes.iter_mut().find(|n| n.id == node_id)
    }

    pub fn get_current_parent(&self, game_id: u32) -> i32 {
        self.data.get(&game_id).map(|g| g.cur).unwrap_or(-1)
    }

    pub fn set_current_parent(&mut self, game_id: u32, parent_id: i32) {
        let game_data = self.get_or_create_game_data(game_id);
        game_data.cur = parent_id;
    }

    pub fn get_children(&self, game_id: u32, parent_id: i32) -> Vec<&Node> {
        self.data.get(&game_id)
            .map(|g| g.nodes.iter().filter(|n| n.parent_id == parent_id).collect())
            .unwrap_or_default()
    }
}
