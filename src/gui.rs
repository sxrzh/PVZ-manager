use eframe::egui;
use egui::{Color32, RichText, TextStyle, Button};
use std::collections::HashMap;
use super::models::ManagerData;
use super::file_io::{
    init_save_path, load_manager_data, save_manager_data, backup_game_file,
    restore_game_file, delete_backup_file, load_game_ids, search_game_id,
};

#[derive(Clone, PartialEq)]
enum ViewMode {
    Table,
    Tree,
}

struct NodeDialog {
    game_id: u32,
    node_id: u32,
    show: bool,
}

struct UserIdDialog {
    user_id: String,
    show: bool,
}

struct ConfirmAction {
    game_id: u32,
    node_id: u32,
    user_id: u32,
    action_type: ConfirmActionType,
}

enum ConfirmActionType {
    Restore,
    Delete,
}

pub struct PVZManagerApp {
    data: ManagerData,
    game_ids: HashMap<String, u32>,
    search_query: String,
    selected_game: Option<u32>,
    view_mode: ViewMode,
    node_dialog: NodeDialog,
    user_id_dialog: UserIdDialog,
    show_confirm: bool,
    confirm_action: Option<ConfirmAction>,
    confirm_message: String,
    message: Option<String>,
}

impl PVZManagerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        init_save_path().unwrap();
        let data = load_manager_data().unwrap();
        let game_ids = load_game_ids().unwrap_or_default();
        
        let user_id_dialog = UserIdDialog {
            user_id: String::new(),
            show: data.user_id.is_none(),
        };

        Self {
            data,
            game_ids,
            search_query: String::new(),
            selected_game: None,
            view_mode: ViewMode::Table,
            node_dialog: NodeDialog { game_id: 0, node_id: 0, show: false },
            user_id_dialog,
            show_confirm: false,
            confirm_action: None,
            confirm_message: String::new(),
            message: None,
        }
    }

    fn get_game_name(&self, game_id: u32) -> String {
        for (name, id) in &self.game_ids {
            if *id == game_id {
                return name.clone();
            }
        }
        format!("Unknown ({})", game_id)
    }

    fn show_message(&mut self, msg: String) {
        self.message = Some(msg);
    }

    fn show_confirm(&mut self, message: String, action: ConfirmAction) {
        self.confirm_message = message;
        self.confirm_action = Some(action);
        self.show_confirm = true;
    }

    fn execute_confirm(&mut self) {
        if let Some(action) = self.confirm_action.take() {
            match action.action_type {
                ConfirmActionType::Restore => {
                    if restore_game_file(action.user_id, action.game_id, action.node_id).unwrap() {
                        self.data.set_current_parent(action.user_id, action.game_id, action.node_id as i32);
                        save_manager_data(&self.data).unwrap();
                        self.show_message("恢复成功".to_string());
                    } else {
                        self.show_message("恢复失败".to_string());
                    }
                }
                ConfirmActionType::Delete => {
                    if delete_backup_file(action.user_id, action.game_id, action.node_id).unwrap() {
                        if let Some(node_mut) = self.data.find_node_mut(action.game_id, action.node_id) {
                            node_mut.file_deleted = true;
                        }
                        save_manager_data(&self.data).unwrap();
                        self.show_message("删除成功".to_string());
                    } else {
                        self.show_message("删除失败".to_string());
                    }
                }
            }
            self.node_dialog.show = false;
        }
        self.show_confirm = false;
    }

    fn render_tree_node(&mut self, ui: &mut egui::Ui, _user_id: u32, game_id: u32, node_id: u32, depth: usize) {
        if let Some(node) = self.data.find_node(game_id, node_id).cloned() {
            let indent = (depth * 20) as f32;
            let children_ids: Vec<u32> = self.data.get_children(game_id, node_id as i32)
                .iter().map(|c| c.id).collect();
            
            ui.horizontal(|ui| {
                ui.add_space(indent);
                let color = match (node.parent_id == -1, node.file_deleted) {
                    (true, _) => Color32::GRAY,
                    (false, true) => Color32::RED,
                    (false, false) => Color32::WHITE,
                };
                ui.label(RichText::new(format!("{} (ID: {})", node.name, node.id)).color(color));
                if ui.button("查看").clicked() {
                    self.node_dialog.game_id = game_id;
                    self.node_dialog.node_id = node.id;
                    self.node_dialog.show = true;
                }
            });
            
            for child_id in children_ids {
                self.render_tree_node(ui, _user_id, game_id, child_id, depth + 1);
            }
        }
    }

    fn show_node_dialog(&mut self, ctx: &egui::Context, user_id: u32) {
        let game_id = self.node_dialog.game_id;
        let node_id = self.node_dialog.node_id;
        
        if let Some(node) = self.data.find_node(game_id, node_id).cloned() {
            let parent_name = if node.parent_id == -1 {
                "无".to_string()
            } else if let Some(parent) = self.data.find_node(game_id, node.parent_id as u32) {
                parent.name.clone()
            } else {
                "未知".to_string()
            };

            egui::Window::new(format!("节点详情 - {}", node.name))
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("节点编号：{}", node.id));
                    ui.label(format!("名称：{}", node.name));
                    ui.label(format!("创建时间：{}", node.created_at.format("%Y-%m-%d %H:%M:%S")));
                    ui.label(format!("备注：{}", node.note));
                    ui.label(format!("父节点：{} (ID: {})", parent_name, node.parent_id));
                    ui.label(format!("文件状态：{}", if node.file_deleted { "已删除" } else { "存在" }));

                    ui.add_space(20.0);

                    if node.parent_id != -1 {
                        if !node.file_deleted {
                            if ui.add(Button::new(RichText::new("恢复到此备份").text_style(TextStyle::Heading))).clicked() {
                                self.show_confirm(
                                    "确定要恢复到此备份吗？这将覆盖当前游戏存档！".to_string(),
                                    ConfirmAction {
                                        game_id,
                                        node_id,
                                        user_id,
                                        action_type: ConfirmActionType::Restore,
                                    }
                                );
                            }
                        }

                        ui.add_space(10.0);

                        let current_parent = self.data.get_current_parent(user_id, game_id);
                        if current_parent != node_id as i32 {
                            if ui.add(Button::new(RichText::new("删除此备份").text_style(TextStyle::Heading))).clicked() {
                                self.show_confirm(
                                    "确定要删除此备份吗？删除后可以恢复，但文件将被移除！".to_string(),
                                    ConfirmAction {
                                        game_id,
                                        node_id,
                                        user_id,
                                        action_type: ConfirmActionType::Delete,
                                    }
                                );
                            }
                        } else {
                            ui.label(RichText::new("当前节点不能删除").color(Color32::RED));
                        }
                    } else {
                        ui.label(RichText::new("根节点不能恢复或删除").color(Color32::RED));
                    }

                    ui.add_space(20.0);

                    if ui.button("关闭").clicked() {
                        self.node_dialog.show = false;
                    }
                });
        }
    }
}

impl eframe::App for PVZManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.user_id_dialog.show {
            egui::Window::new("设置用户编号")
                .fixed_size([300.0, 150.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("请输入您的游戏用户编号（从1开始）：");
                    ui.text_edit_singleline(&mut self.user_id_dialog.user_id);
                    ui.add_space(10.0);
                    if ui.button("确定").clicked() {
                        if let Ok(user_id) = self.user_id_dialog.user_id.parse::<u32>() {
                            if user_id >= 1 {
                                self.data.user_id = Some(user_id);
                                save_manager_data(&self.data).unwrap();
                                self.user_id_dialog.show = false;
                                self.show_message(format!("用户编号已设置为 {}", user_id));
                            } else {
                                self.show_message("用户编号必须大于0".to_string());
                            }
                        } else {
                            self.show_message("请输入有效的数字".to_string());
                        }
                    }
                });
            return;
        }

        if self.show_confirm {
            egui::Window::new("确认")
                .fixed_size([300.0, 150.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&self.confirm_message);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            self.show_confirm = false;
                            self.confirm_action = None;
                        }
                        if ui.button("确认").clicked() {
                            self.execute_confirm();
                        }
                    });
                });
            return;
        }

        if self.message.is_some() {
            let msg = self.message.clone().unwrap();
            egui::Window::new("提示")
                .fixed_size([300.0, 100.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(&msg);
                    if ui.button("确定").clicked() {
                        self.message = None;
                    }
                });
            return;
        }

        let user_id = match self.data.user_id {
            Some(id) => id,
            None => return,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("PVZ 存档管理器 - 用户 {}", user_id));
            ui.add_space(10.0);

            ui.text_edit_singleline(&mut self.search_query);
            let results = search_game_id(&self.game_ids, &self.search_query);
            
            if !results.is_empty() {
                ui.group(|ui| {
                    ui.label("搜索结果：");
                    for (name, id) in &results {
                        if ui.button(format!("{} (ID: {})", name, id)).clicked() {
                            self.selected_game = Some(*id);
                            self.search_query.clear();
                        }
                    }
                });
            }

            ui.add_space(10.0);

            if self.selected_game.is_none() {
                ui.label("请从上方搜索框输入游戏名称或编号进行选择");
                return;
            }

            let game_id = self.selected_game.unwrap();
            ui.label(format!("当前游戏：{} (ID: {})", self.get_game_name(game_id), game_id));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.selectable_label(self.view_mode == ViewMode::Table, "表格模式").clicked() {
                    self.view_mode = ViewMode::Table;
                }
                if ui.selectable_label(self.view_mode == ViewMode::Tree, "树形模式").clicked() {
                    self.view_mode = ViewMode::Tree;
                }
                if ui.button("备份当前存档").clicked() {
                    let parent_id = self.data.get_current_parent(user_id, game_id);
                    let node_id = self.data.create_node(user_id, game_id, parent_id, "未命名备份".to_string(), String::new());
                    if backup_game_file(user_id, game_id, node_id).unwrap() {
                        self.data.set_current_parent(user_id, game_id, node_id as i32);
                        save_manager_data(&self.data).unwrap();
                        self.show_message(format!("备份成功（ID: {}）", node_id));
                    } else {
                        self.show_message("备份失败：游戏文件不存在".to_string());
                    }
                }
            });

            ui.add_space(10.0);

            match self.view_mode {
                ViewMode::Table => {
                    let nodes: Vec<_> = self.data.data.get(&game_id)
                        .map(|g| g.nodes.iter().filter(|n| n.parent_id != -1 && !n.file_deleted).cloned().collect())
                        .unwrap_or_default();

                    if nodes.is_empty() {
                        ui.label("暂无存档备份");
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for node in nodes {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{}", node.id));
                                        ui.label(node.name.clone());
                                        ui.label(node.created_at.format("%Y-%m-%d %H:%M:%S").to_string());
                                        if !node.note.is_empty() {
                                            ui.label(RichText::new(&node.note).color(Color32::GREEN));
                                        }
                                    });
                                    if ui.button("查看详情").clicked() {
                                        self.node_dialog.game_id = game_id;
                                        self.node_dialog.node_id = node.id;
                                        self.node_dialog.show = true;
                                    }
                                });
                            }
                        });
                    }
                }
                ViewMode::Tree => {
                    let root_id = self.data.get_root_id(user_id, game_id);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.render_tree_node(ui, user_id, game_id, root_id, 0);
                    });
                }
            }
        });

        if self.node_dialog.show {
            self.show_node_dialog(ctx, user_id);
        }
    }
}
