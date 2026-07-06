#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod file_io;
mod cli;
mod gui;

use std::env;
use std::sync::Arc;

// 将图标嵌入到程序中
const ICON_BYTES: &[u8] = include_bytes!("../blobs/pvz-manager-icon.ico");

fn load_icon() -> Option<Arc<egui::IconData>> {
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(ICON_BYTES)).ok()?;
    
    // 找到最大的图标条目
    let entry = icon_dir.entries().iter()
        .max_by_key(|e| e.width())?;
    
    // 解码图标
    let image = entry.decode().ok()?;
    let width = image.width();
    let height = image.height();
    let rgba = image.rgba_data();
    
    Some(Arc::new(egui::IconData {
        rgba: rgba.to_vec(),
        width,
        height,
    }))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"-c".to_string()) {
        if let Err(e) = cli::run_cli() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        // 加载窗口图标
        let icon = load_icon();
        
        let mut viewport = egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(1100.0, 800.0));
        if let Some(icon) = icon {
            viewport = viewport.with_icon(icon);
        }
        
        let native_options = eframe::NativeOptions {
            viewport,
            ..eframe::NativeOptions::default()
        };
        if let Err(e) = eframe::run_native(
            "《植物大战僵尸》存档管理器 v0.1",
            native_options,
            Box::new(|cc| Ok(Box::new(gui::PVZManagerApp::new(cc)))),
        ) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
