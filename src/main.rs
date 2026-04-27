mod network;
mod qr;
mod server;

use clap::Parser;
use std::path::Path;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(author, version, about = "Local AirDrop via CLI")]
struct Args {
    /// The path to the file you want to share
    file: String,

    /// Optional port to run the server on (default: 8080)
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 1. Verify the file actually exists before doing anything
    if !Path::new(&args.file).exists() {
        eprintln!("❌ Error: File '{}' does not exist.", args.file);
        exit(1);
    }

    // 2. Fetch the local IP address
    let ip = match network::get_local_ip() {
        Some(ip) => ip,
        None => {
            eprintln!("❌ Error: Could not determine local IP address. Are you connected to Wi-Fi?");
            exit(1);
        }
    };

    // 3. Construct the download URL (Notice the 'https')
    let download_url = format!("https://{}:{}/download", ip, args.port);
    
    // 4. Print the CLI Interface
    println!("\n🚀 [lan-drop] Starting local server...");
    println!("📄 [lan-drop] File: {}", args.file);
    println!("🌐 [lan-drop] Serving at: {}\n", download_url);

    // 5. Render the QR Code
    qr::print_url_qr(&download_url);
    println!("\n📱 Scan the QR code above with your phone to download.");
    println!("🛑 Press Ctrl+C to stop the server.\n");

    // 6. Start the Axum Server (This blocks the thread and keeps the app running)
    server::start(ip, args.port, args.file).await;
}