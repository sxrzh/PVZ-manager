mod models;
mod file_io;
mod cli;
mod gui;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"-c".to_string()) {
        if let Err(e) = cli::run_cli() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    } else {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(900.0, 600.0)),
            ..eframe::NativeOptions::default()
        };
        if let Err(e) = eframe::run_native(
            "PVZ 存档管理器",
            native_options,
            Box::new(|cc| Box::new(gui::PVZManagerApp::new(cc))),
        ) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
