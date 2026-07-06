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
                name: "空".to_string(),
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
        let existing_ids: std::collections::HashSet<_> = game_data.nodes.iter().map(|n| n.id).collect();
        let mut node_id = 1 as u32;
        while existing_ids.contains(&node_id) {
            node_id += 1;
        }
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

    /// 构建邻接表，用于快速查找子节点
    pub fn build_adjacency_list(&self, game_id: u32) -> std::collections::HashMap<i32, Vec<&Node>> {
        let mut adj: std::collections::HashMap<i32, Vec<&Node>> = std::collections::HashMap::new();
        if let Some(game_data) = self.data.get(&game_id) {
            for node in &game_data.nodes {
                adj.entry(node.parent_id).or_default().push(node);
            }
        }
        adj
    }

    /// 递归计算节点的 flag（是否应该保留）
    fn calculate_flags(
        &self,
        game_id: u32,
        node_id: u32,
        adj: &std::collections::HashMap<i32, Vec<&Node>>,
        current_parent: i32,
        flags: &mut std::collections::HashMap<u32, bool>,
    ) {
        // 先处理子节点
        if let Some(children) = adj.get(&(node_id as i32)) {
            for child in children {
                self.calculate_flags(game_id, child.id, adj, current_parent, flags);
            }
        }

        let mut flag = false;

        if let Some(node) = self.find_node(game_id, node_id) {
            // 根节点、当前存档节点、文件未删除的节点应该保留
            if node.id == 0 || (current_parent != -1 && node.id == current_parent as u32) || !node.file_deleted {
                flag = true;
            } else {
                // 如果有子节点 flag 为 true，则该节点也应该保留
                if let Some(children) = adj.get(&(node_id as i32)) {
                    for child in children {
                        if *flags.get(&child.id).unwrap_or(&false) {
                            flag = true;
                            break;
                        }
                    }
                }
            }
        }

        flags.insert(node_id, flag);
    }

    /// 清理无用节点，返回被删除的节点 ID 列表
    pub fn clean_empty_nodes(&mut self, game_id: u32) -> Vec<u32> {
        if !self.data.contains_key(&game_id) {
            return Vec::new();
        }

        let current_parent = self.get_current_parent(game_id);
        let adj_list = self.build_adjacency_list(game_id);

        let mut flags: std::collections::HashMap<u32, bool> = std::collections::HashMap::new();
        self.calculate_flags(game_id, 0, &adj_list, current_parent, &mut flags);

        // 收集需要删除的节点（flag 为 false 且不是根节点）
        let to_delete: Vec<u32> = flags.iter()
            .filter(|(node_id, flag)| !*flag && **node_id != 0)
            .map(|(node_id, _)| *node_id)
            .collect();

        if to_delete.is_empty() {
            return Vec::new();
        }

        // 从 nodes 中移除这些节点
        if let Some(game_data) = self.data.get_mut(&game_id) {
            game_data.nodes.retain(|n| !to_delete.contains(&n.id));
        }

        to_delete
    }
}
