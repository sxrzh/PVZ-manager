use anyhow::Result;
use clap::{Parser, Subcommand};
use super::models::ManagerData;
use super::file_io::{
    init_save_path, load_manager_data, save_manager_data, backup_game_file,
    restore_game_file, delete_backup_file, delete_game_file, load_game_ids, search_game_id,
    get_backup_file_path, check_game_file_exists, get_game_file_path,
};

#[derive(Parser)]
#[command(name = "pvz-manager", about = "PVZ save file manager")]
struct Cli {
    #[arg(short, long, help = "Enable CLI mode")]
    cli: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    SetUser { user_id: u32 },
    ListGames,
    SearchGames { query: String },
    ListNodes { game_id: u32 },
    Backup { game_id: u32, name: String, note: String },
    Restore { game_id: u32, node_id: u32 },
    Delete { game_id: u32, node_id: u32 },
    Rename { game_id: u32, node_id: u32, name: String },
    ShowNode { game_id: u32, node_id: u32 },
    ShowTree { game_id: u32 },
    ShowCurrent { game_id: u32 },
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    if !cli.cli {
        return Ok(());
    }

    init_save_path()?;
    let mut data = load_manager_data()?;

    if let Some(Commands::SetUser { user_id }) = &cli.command {
        data.user_id = Some(*user_id);
        save_manager_data(&data)?;
        println!("User ID set to: {}", user_id);
        return Ok(());
    }

    let user_id = if let Some(id) = data.user_id {
        id
    } else {
        println!("No user ID set. Please run 'pvz-manager -c set-user <user_id>' first.");
        return Ok(());
    };

    match cli.command {
        Some(Commands::SetUser { user_id: _ }) => {
        }
        Some(Commands::ListGames) => {
            let game_ids = load_game_ids()?;
            println!("Available games:");
            for (name, id) in game_ids {
                println!("  {} (ID: {})", name, id);
            }
        }
        Some(Commands::SearchGames { query }) => {
            let game_ids = load_game_ids()?;
            let results = search_game_id(&game_ids, &query);
            println!("Search results for '{}':", query);
            for (name, id) in results {
                println!("  {} (ID: {})", name, id);
            }
        }
        Some(Commands::ListNodes { game_id }) => {
            data.get_root_id(game_id);
            println!("Nodes for game {}:", game_id);
            for node in data.get_children(game_id, -1) {
                print_node_recursive(&data, game_id, node.id, 0);
            }
        }
        Some(Commands::Backup { game_id, name, note }) => {
            data.get_root_id(game_id);
            let parent_id = data.get_current_parent(game_id);
            let node_id = data.create_node(game_id, parent_id, name, note);
            if backup_game_file(user_id, game_id, node_id)? {
                data.set_current_parent(game_id, node_id as i32);
                save_manager_data(&data)?;
                println!("Backup created successfully (ID: {})", node_id);
            } else {
                println!("Failed: Game file does not exist");
            }
        }
        Some(Commands::Restore { game_id, node_id }) => {
            if let Some(node) = data.find_node(game_id, node_id) {
                if node.file_deleted {
                    println!("Failed: Backup file has been deleted");
                    return Ok(());
                }
                if node_id == 0 {
                    if delete_game_file(user_id, game_id)? {
                        data.set_current_parent(game_id, node_id as i32);
                        save_manager_data(&data)?;
                        println!("Successfully restored to backup {} (root)", node_id);
                    }
                    else {
                        println!("Successfully: Game file does not exist");
                    }
                }
                else if restore_game_file(user_id, game_id, node_id)? {
                    data.set_current_parent(game_id, node_id as i32);
                    save_manager_data(&data)?;
                    println!("Successfully restored to backup {}", node_id);
                } else {
                    println!("Failed: Backup file does not exist");
                }
            } else {
                println!("Failed: Node not found");
            }
        }
        Some(Commands::Delete { game_id, node_id }) => {
            if let Some(node) = data.find_node(game_id, node_id) {
                if node.parent_id == -1 {
                    println!("Failed: Cannot delete root node");
                    return Ok(());
                }
                if delete_backup_file(user_id, game_id, node_id)? {
                    if let Some(node_mut) = data.find_node_mut(game_id, node_id) {
                        node_mut.file_deleted = true;
                    }
                    save_manager_data(&data)?;
                    println!("Backup file deleted (ID: {})", node_id);
                } else {
                    println!("Failed: Backup file does not exist");
                }
            } else {
                println!("Failed: Node not found");
            }
        }
        Some(Commands::Rename { game_id, node_id, name }) => {
            if let Some(node) = data.find_node(game_id, node_id) {
                if node.parent_id == -1 {
                    println!("Failed: Cannot rename root node");
                    return Ok(());
                }
                if let Some(node_mut) = data.find_node_mut(game_id, node_id) {
                    node_mut.name = name;
                    save_manager_data(&data)?;
                    println!("Node renamed successfully");
                }
            } else {
                println!("Failed: Node not found");
            }
        }
        Some(Commands::ShowNode { game_id, node_id }) => {
            if let Some(node) = data.find_node(game_id, node_id) {
                let parent_name = if node.parent_id == -1 {
                    "无".to_string()
                } else if let Some(parent) = data.find_node(game_id, node.parent_id as u32) {
                    parent.name.clone()
                } else {
                    "未知".to_string()
                };
                println!("Node ID: {}", node.id);
                println!("Name: {}", node.name);
                println!("Parent: {} (ID: {})", parent_name, node.parent_id);
                println!("Created at: {}", node.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!("Note: {}", node.note);
                println!("File deleted: {}", node.file_deleted);
                println!("Backup file: {}", get_backup_file_path(user_id, game_id, node_id).display());
            } else {
                println!("Failed: Node not found");
            }
        }
        Some(Commands::ShowTree { game_id }) => {
            println!("Version tree for game {}:", game_id);
            let root_id = data.get_root_id(game_id);
            if let Some(root) = data.find_node(game_id, root_id) {
                print_tree_node(&data, game_id, root, 0);
            }
            let current_parent = data.get_current_parent(game_id);
            if current_parent != -1 {
                if let Some(cur_node) = data.find_node(game_id, current_parent as u32) {
                    println!("\nCurrent: -> {} (ID: {})", cur_node.name, current_parent);
                }
            }
            if check_game_file_exists(user_id, game_id) {
                println!("Game file exists: {}", get_game_file_path(user_id, game_id).display());
            }
        }
        Some(Commands::ShowCurrent { game_id }) => {
            let current_parent = data.get_current_parent(game_id);
            if current_parent == -1 {
                println!("No current save or current parent is root");
            } else if let Some(node) = data.find_node(game_id, current_parent as u32) {
                println!("Current parent: {} (ID: {})", node.name, current_parent);
                println!("Game file exists: {}", check_game_file_exists(user_id, game_id));
            } else {
                println!("Current parent node not found");
            }
        }
        None => {
            println!("No command specified. Use --help for available commands.");
        }
    }

    Ok(())
}

fn print_node_recursive(data: &ManagerData, game_id: u32, node_id: u32, depth: usize) {
    let adj_list = build_adjacency_list(data, game_id);
    print_node_recursive_with_adj(data, game_id, node_id, depth, &adj_list);
}

fn print_node_recursive_with_adj(data: &ManagerData, game_id: u32, node_id: u32, depth: usize, adj: &std::collections::HashMap<i32, Vec<&super::models::Node>>) {
    if let Some(node) = data.find_node(game_id, node_id) {
        let prefix = "  ".repeat(depth);
        let status = if node.file_deleted { "[DELETED]" } else { "" };
        println!("{}{} {} (ID: {})", prefix, status, node.name, node.id);
        if let Some(children) = adj.get(&(node_id as i32)) {
            for child in children {
                print_node_recursive_with_adj(data, game_id, child.id, depth + 1, adj);
            }
        }
    }
}

fn print_tree_node(data: &ManagerData, game_id: u32, node: &super::models::Node, depth: usize) {
    let adj_list = build_adjacency_list(data, game_id);
    print_tree_node_with_adj(data, game_id, node, depth, &adj_list);
}

fn build_adjacency_list(data: &ManagerData, game_id: u32) -> std::collections::HashMap<i32, Vec<&super::models::Node>> {
    let mut adj = std::collections::HashMap::new();
    if let Some(game_data) = data.data.get(&game_id) {
        for node in &game_data.nodes {
            adj.entry(node.parent_id).or_insert_with(Vec::new).push(node);
        }
    }
    adj
}

fn print_tree_node_with_adj(data: &ManagerData, game_id: u32, node: &super::models::Node, depth: usize, adj: &std::collections::HashMap<i32, Vec<&super::models::Node>>) {
    let prefix = "  ".repeat(depth);
    let status = match (node.parent_id == -1, node.file_deleted) {
        (true, _) => "[ROOT]",
        (false, true) => "[DELETED]",
        (false, false) => "",
    };
    println!("{}{} {} (ID: {})", prefix, status, node.name, node.id);
    if let Some(children) = adj.get(&(node.id as i32)) {
        for child in children {
            print_tree_node_with_adj(data, game_id, child, depth + 1, adj);
        }
    }
}
