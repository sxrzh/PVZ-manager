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
    pub roots: std::collections::HashMap<u32, u32>,
    pub nodes: Vec<Node>,
    pub cur: std::collections::HashMap<u32, i32>,
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
        self.data.entry(game_id).or_insert_with(|| GameData {
            roots: std::collections::HashMap::new(),
            nodes: Vec::new(),
            cur: std::collections::HashMap::new(),
        })
    }

    pub fn get_root_id(&mut self, user_id: u32, game_id: u32) -> u32 {
        let game_data = self.get_or_create_game_data(game_id);
        *game_data.roots.entry(user_id).or_insert_with(|| {
            let root_id = game_data.nodes.len() as u32;
            game_data.nodes.push(Node {
                id: root_id,
                name: "根节点".to_string(),
                parent_id: -1,
                created_at: chrono::Local::now(),
                note: "根节点，表示没有存档".to_string(),
                file_deleted: false,
            });
            game_data.cur.insert(user_id, root_id as i32);
            root_id
        })
    }

    pub fn create_node(&mut self, _user_id: u32, game_id: u32, parent_id: i32, name: String, note: String) -> u32 {
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

    pub fn get_current_parent(&self, user_id: u32, game_id: u32) -> i32 {
        self.data.get(&game_id).and_then(|g| g.cur.get(&user_id)).copied().unwrap_or(-1)
    }

    pub fn set_current_parent(&mut self, user_id: u32, game_id: u32, parent_id: i32) {
        let game_data = self.get_or_create_game_data(game_id);
        game_data.cur.insert(user_id, parent_id);
    }

    pub fn get_children(&self, game_id: u32, parent_id: i32) -> Vec<&Node> {
        self.data.get(&game_id)
            .map(|g| g.nodes.iter().filter(|n| n.parent_id == parent_id).collect())
            .unwrap_or_default()
    }

    pub fn has_children(&self, game_id: u32, node_id: u32) -> bool {
        !self.get_children(game_id, node_id as i32).is_empty()
    }

    pub fn all_nodes_have_deleted_files(&self, game_id: u32, node_id: u32) -> bool {
        if let Some(node) = self.find_node(game_id, node_id) {
            if !node.file_deleted {
                return false;
            }
            for child in self.get_children(game_id, node_id as i32) {
                if !self.all_nodes_have_deleted_files(game_id, child.id) {
                    return false;
                }
            }
            true
        } else {
            true
        }
    }
}
