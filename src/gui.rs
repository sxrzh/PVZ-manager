use eframe::egui;
use egui::{Color32, RichText, TextStyle, Button, FontDefinitions};
use std::collections::HashMap;
use super::models::ManagerData;
use super::file_io::{
    init_save_path, load_manager_data, save_manager_data, backup_game_file,
    restore_game_file, delete_backup_file, load_game_ids, search_game_id,
    check_game_file_exists,
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
    editing: bool,
    edit_name: String,
    edit_note: String,
}

struct UserIdDialog {
    user_id: String,
    show: bool,
}

struct BackupDialog {
    game_id: u32,
    parent_id: i32,
    name: String,
    note: String,
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
    backup_dialog: BackupDialog,
    show_confirm: bool,
    confirm_action: Option<ConfirmAction>,
    confirm_message: String,
    message: Option<String>,
}

impl PVZManagerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        init_save_path().unwrap();
        let data = load_manager_data().unwrap();
        let game_ids = load_game_ids();
        
        let mut fonts = FontDefinitions::default();
        let msyh_path = "C:\\Windows\\Fonts\\msyh.ttc";
        if let Ok(font_bytes) = std::fs::read(msyh_path) {
            fonts.font_data.insert(
                "msyh".to_string(),
                egui::FontData::from_owned(font_bytes).into(),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("msyh".to_string());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("msyh".to_string());
        }
        cc.egui_ctx.set_fonts(fonts);

        let mut style = (*cc.egui_ctx.style()).clone();
        // style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(64.0));
        // style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(44.0));
        // style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(44.0));
        // style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(40.0));

        style.visuals = egui::Visuals::light();
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 120, 200);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(80, 140, 220);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(60, 120, 200);

        cc.egui_ctx.set_style(std::sync::Arc::new(style));
        
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
            node_dialog: NodeDialog { game_id: 0, node_id: 0, show: false, editing: false, edit_name: String::new(), edit_note: String::new() },
            user_id_dialog,
            backup_dialog: BackupDialog { game_id: 0, parent_id: 0, name: String::new(), note: String::new(), show: false },
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
                        self.data.set_current_parent(action.game_id, action.node_id as i32);
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
                        // 删除后清理无用节点
                        let deleted_nodes = self.data.clean_empty_nodes(action.game_id);
                        for node_id in &deleted_nodes {
                            delete_backup_file(action.user_id, action.game_id, *node_id).unwrap();
                        }
                        save_manager_data(&self.data).unwrap();
                        if deleted_nodes.is_empty() {
                            self.show_message("删除成功".to_string());
                        } else {
                            self.show_message(format!("删除成功，同时清理了 {} 个无用节点", deleted_nodes.len()));
                        }
                    } else {
                        self.show_message("删除失败".to_string());
                    }
                }
            }
            self.node_dialog.show = false;
        }
        self.show_confirm = false;
    }

    fn render_tree_node(&mut self, ui: &mut egui::Ui, game_id: u32, node_id: u32, depth: usize) {
        let adj_list = self.build_adjacency_list(game_id);
        self.render_tree_node_with_adj(ui, game_id, node_id, depth, &adj_list);
    }

    fn build_adjacency_list(&self, game_id: u32) -> std::collections::HashMap<i32, Vec<u32>> {
        let mut adj = std::collections::HashMap::new();
        if let Some(game_data) = self.data.data.get(&game_id) {
            for node in &game_data.nodes {
                adj.entry(node.parent_id).or_insert_with(Vec::new).push(node.id);
            }
        }
        adj
    }

    fn render_tree_node_with_adj(&mut self, ui: &mut egui::Ui, game_id: u32, node_id: u32, depth: usize, adj: &std::collections::HashMap<i32, Vec<u32>>) {
        if let Some(node) = self.data.find_node(game_id, node_id).cloned() {
            let indent = (depth * 20) as f32;
            
            ui.horizontal(|ui| {
                ui.add_space(indent);
                let color = match (node.parent_id == -1, node.file_deleted) {
                    (true, _) => Color32::GRAY,
                    (false, true) => Color32::RED,
                    (false, false) => Color32::BLACK,
                };
                ui.label(RichText::new(format!("{} (ID: {})", node.name, node.id)).color(color));
                if ui.button("查看").clicked() {
                    self.node_dialog.game_id = game_id;
                    self.node_dialog.node_id = node.id;
                    self.node_dialog.show = true;
                }
            });
            
            if let Some(children) = adj.get(&(node_id as i32)) {
                for child_id in children {
                    self.render_tree_node_with_adj(ui, game_id, *child_id, depth + 1, adj);
                }
            }
            
            let current_parent = self.data.get_current_parent(game_id);
            if current_parent == node_id as i32 {
                if let Some(user_id) = self.data.user_id {
                    if check_game_file_exists(user_id, game_id) {
                        let current_indent = ((depth + 1) * 20) as f32;
                        ui.horizontal(|ui| {
                            ui.add_space(current_indent);
                            ui.label(RichText::new("当前存档").color(Color32::GREEN));
                        });
                    }
                }
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

            if !self.node_dialog.editing {
                self.node_dialog.edit_name = node.name.clone();
                self.node_dialog.edit_note = node.note.clone();
            }

            egui::Window::new(format!("备份详情 - {}", node.name))
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("节点编号：{}", node.id));
                    
                    if self.node_dialog.editing {
                        ui.label("名称：");
                        ui.text_edit_singleline(&mut self.node_dialog.edit_name);
                        ui.label("备注：");
                        ui.text_edit_multiline(&mut self.node_dialog.edit_note);
                    } else {
                        ui.label(format!("名称：{}", node.name));
                        ui.label(format!("创建时间：{}", node.created_at.format("%Y-%m-%d %H:%M:%S")));
                        ui.label(format!("备注：{}", node.note));
                    }
                    
                    ui.label(format!("修改自：{} (ID: {})", parent_name, node.parent_id));
                    ui.label(format!("文件状态：{}", if node.file_deleted { "已删除" } else { "存在" }));

                    ui.add_space(20.0);

                    if self.node_dialog.editing {
                        ui.horizontal(|ui| {
                            if ui.button("保存").clicked() {
                                if let Some(node_mut) = self.data.find_node_mut(game_id, node_id) {
                                    node_mut.name = self.node_dialog.edit_name.clone();
                                    node_mut.note = self.node_dialog.edit_note.clone();
                                }
                                save_manager_data(&self.data).unwrap();
                                self.node_dialog.editing = false;
                                self.show_message("修改成功".to_string());
                            }
                            if ui.button("取消").clicked() {
                                self.node_dialog.editing = false;
                            }
                        });
                    } else {
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

                            if ui.add(Button::new(RichText::new("修改名称/备注").text_style(TextStyle::Heading))).clicked() {
                                self.node_dialog.editing = true;
                            }

                            ui.add_space(10.0);

                            if ui.add(Button::new(RichText::new("删除此备份").text_style(TextStyle::Heading))).clicked() {
                                self.show_confirm(
                                    "确定要删除此备份吗？".to_string(),
                                    ConfirmAction {
                                        game_id,
                                        node_id,
                                        user_id,
                                        action_type: ConfirmActionType::Delete,
                                    }
                                );
                            }
                        } else {
                            ui.label(RichText::new("此节点不能恢复或删除").color(Color32::RED));
                            
                            ui.add_space(10.0);
                            
                            if ui.add(Button::new(RichText::new("修改名称/备注").text_style(TextStyle::Heading))).clicked() {
                                self.node_dialog.editing = true;
                            }
                        }
                    }

                    ui.add_space(20.0);

                    if !self.node_dialog.editing && ui.button("关闭").clicked() {
                        self.node_dialog.show = false;
                    }
                });
        }
    }
}

impl eframe::App for PVZManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 每帧设置样式，确保字体大小生效
        let mut style = (*ctx.style()).clone();

        // 条件编译：large-font feature 使用大字体，否则使用原字体大小
        #[cfg(feature = "large-font")]
        {
            style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(64.0));
            style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(44.0));
            style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(44.0));
            style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(40.0));
        }

        #[cfg(not(feature = "large-font"))]
        {
            style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(32.0));
            style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(22.0));
            style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(22.0));
            style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(20.0));
        }

        ctx.set_style(std::sync::Arc::new(style));

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

        if self.backup_dialog.show {
            let game_id = self.backup_dialog.game_id;
            let parent_id = self.backup_dialog.parent_id;
            egui::Window::new("备份当前存档")
                .fixed_size([400.0, 200.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("请输入备份名称和备注：");
                    ui.add_space(10.0);
                    ui.label("名称：");
                    ui.text_edit_singleline(&mut self.backup_dialog.name);
                    ui.label("备注：");
                    ui.text_edit_singleline(&mut self.backup_dialog.note);
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            self.backup_dialog.show = false;
                        }
                        if ui.button("备份").clicked() {
                            let name = if self.backup_dialog.name.is_empty() {
                                "未命名备份".to_string()
                            } else {
                                self.backup_dialog.name.clone()
                            };
                            let note = self.backup_dialog.note.clone();
                            let node_id = self.data.create_node(game_id, parent_id, name, note);
                            if let Some(user_id) = self.data.user_id {
                                if backup_game_file(user_id, game_id, node_id).unwrap() {
                                    self.data.set_current_parent(game_id, node_id as i32);
                                    save_manager_data(&self.data).unwrap();
                                    self.show_message(format!("备份成功（ID: {}）", node_id));
                                } else {
                                    self.show_message("备份失败：游戏文件不存在".to_string());
                                }
                            }
                            self.backup_dialog.show = false;
                        }
                    });
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
            ui.heading(format!("《植物大战僵尸》存档管理器 - 用户 {}", user_id));
            ui.add_space(10.0);

            let available_height = ui.available_height();
            let top_height = available_height * (1.0 / 3.0);
            let bottom_height = available_height * (2.0 / 3.0);

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), top_height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.text_edit_singleline(&mut self.search_query);
                    let results = search_game_id(&self.game_ids, &self.search_query);
                    
                    egui::ScrollArea::vertical().id_salt("search_results_scroll").max_height(top_height - 60.0).show(ui, |ui| {
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
                    });
                },
            );

            ui.separator();
            ui.add_space(5.0);

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), bottom_height),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    if self.selected_game.is_none() {
                        ui.label("请从上方搜索框输入游戏名称或编号进行选择");
                    } else {
                        let game_id = self.selected_game.unwrap();
                        ui.label(format!("当前游戏：{} (ID: {})", self.get_game_name(game_id), game_id));

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.selectable_label(self.view_mode == ViewMode::Table, "表格模式").clicked() {
                                self.view_mode = ViewMode::Table;
                            }
                            if ui.selectable_label(self.view_mode == ViewMode::Tree, "树形模式").clicked() {
                                self.view_mode = ViewMode::Tree;
                                // 切换到树形模式时清理无用节点
                                let deleted_nodes = self.data.clean_empty_nodes(game_id);
                                for node_id in &deleted_nodes {
                                    delete_backup_file(user_id, game_id, *node_id).unwrap();
                                }
                                if !deleted_nodes.is_empty() {
                                    save_manager_data(&self.data).unwrap();
                                    self.show_message(format!("清理了 {} 个无用节点", deleted_nodes.len()));
                                }
                            }
                            if check_game_file_exists(user_id, game_id) {
                                if ui.button("备份当前存档").clicked() {
                                    let parent_id = self.data.get_current_parent(game_id);
                                    self.backup_dialog = BackupDialog {
                                        game_id,
                                        parent_id,
                                        name: String::new(),
                                        note: String::new(),
                                        show: true,
                                    };
                                }
                            }
                        });

                        ui.add_space(10.0);

                        egui::ScrollArea::vertical().id_salt("backup_list_scroll").max_height(bottom_height - 100.0).show(ui, |ui| {
                            match self.view_mode {
                                ViewMode::Table => {
                                    let nodes: Vec<_> = self.data.data.get(&game_id)
                                        .map(|g| g.nodes.iter().filter(|n| n.parent_id != -1 && !n.file_deleted).cloned().collect())
                                        .unwrap_or_default();

                                    if nodes.is_empty() {
                                        ui.label("暂无存档备份");
                                    } else {
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
                                    }
                                }
                                ViewMode::Tree => {
                                    let root_id = self.data.get_root_id(game_id);
                                    self.render_tree_node(ui, game_id, root_id, 0);
                                }
                            }
                        });
                    }
                },
            );
        });

        if self.node_dialog.show {
            self.show_node_dialog(ctx, user_id);
        }
    }
}
