mod network;
mod qr;
mod server;

use clap::Parser;
use server::SharedFile;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};
use tokio::sync::RwLock;

#[derive(Parser, Debug)]
#[command(author, version, about = "Local AirDrop via CLI")]
struct Args {
    /// Optional path to the first file you want to share
    file: Option<String>,

    /// Optional port to run the server on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let ip = match network::get_local_ip() {
        Some(ip) => ip,
        None => {
            eprintln!("Error: Could not determine local IP address. Are you connected to Wi-Fi?");
            wait_before_exit();
            exit(1);
        }
    };

    println!();
    println!("[lan-drop] Local file drop is ready.");
    println!("[lan-drop] Type a file path to generate a QR download link.");
    println!("[lan-drop] Type q and press Enter to quit.");

    let first_file = match args.file {
        Some(file) => match validate_file(&file) {
            Ok(path) => path,
            Err(message) => {
                eprintln!("{message}");
                match prompt_for_file() {
                    Some(path) => path,
                    None => return,
                }
            }
        },
        None => match prompt_for_file() {
            Some(path) => path,
            None => return,
        },
    };

    let shared_file: SharedFile = Arc::new(RwLock::new(first_file));
    let download_url = format!("https://{}:{}/download", ip, args.port);
    let server_file = Arc::clone(&shared_file);

    tokio::spawn(async move {
        server::start(ip, args.port, server_file).await;
    });

    show_current_file(&download_url, &shared_file).await;

    loop {
        println!();
        match prompt_for_file() {
            Some(path) => {
                *shared_file.write().await = path;
                show_current_file(&download_url, &shared_file).await;
            }
            None => break,
        }
    }
}

async fn show_current_file(download_url: &str, shared_file: &SharedFile) {
    let file_path = shared_file.read().await.clone();

    println!();
    println!("[lan-drop] File: {}", file_path.display());
    println!("[lan-drop] Serving at: {download_url}");
    println!();
    qr::print_url_qr(download_url);
    println!();
    println!("Scan the QR code above with your phone to download.");
    println!("Leave this window open while the download is running.");
}

fn prompt_for_file() -> Option<PathBuf> {
    loop {
        print!("\nEnter file path (or q to quit): ");
        io::stdout().flush().ok()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        let input = input.trim().trim_matches('"');

        if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
            return None;
        }

        match validate_file(input) {
            Ok(path) => return Some(path),
            Err(message) => eprintln!("{message}"),
        }
    }
}

fn validate_file(input: &str) -> Result<PathBuf, String> {
    if input.is_empty() {
        return Err("Error: Please enter a file path.".to_string());
    }

    let path = Path::new(input);
    if !path.exists() {
        return Err(format!("Error: File '{}' does not exist.", input));
    }

    if !path.is_file() {
        return Err(format!("Error: '{}' is not a file.", input));
    }

    Ok(path.to_path_buf())
}

fn wait_before_exit() {
    println!("Press Enter to close.");
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}
