use anyhow::Result;
use clap::{Parser, Subcommand};

mod api;

use crate::api::MihomoClient;

/// A CLI tool for interacting with Mihomo API
#[derive(Parser, Debug)]
#[command(name = "yact")]
#[command(author = "yact Developer")]
#[command(version = "0.1.0")]
#[command(about = "CLI tool for Mihomo API", long_about = None)]
struct Args {
    /// Mihomo API URL
    #[arg(short, long, default_value = "http://127.0.0.1:9097")]
    url: String,

    /// API Secret key
    #[arg(short, long, default_value = "123456")]
    secret: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get current configuration
    #[command(name = "configs")]
    Configs,

    /// Get all proxies and groups
    #[command(name = "proxies")]
    Proxies,

    /// Get logs (supports SSE streaming)
    #[command(name = "logs")]
    Logs {
        /// Filter by log level (info, warning, error, debug)
        #[arg(short, long)]
        level: Option<String>,

        /// Follow logs continuously
        #[arg(short, long)]
        follow: bool,
    },

    /// Get version information
    #[command(name = "version")]
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = MihomoClient::new(&args.url, args.secret.as_deref().unwrap_or(""));

    match &args.command {
        Commands::Configs => match client.get_configs().await {
            Ok(configs) => {
                println!("{}", serde_json::to_string_pretty(&configs)?);
            }
            Err(e) => {
                eprintln!("Failed to get configs: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Proxies => match client.get_proxies().await {
            Ok(proxies) => {
                println!("{}", serde_json::to_string_pretty(&proxies)?);
            }
            Err(e) => {
                eprintln!("Failed to get proxies: {}", e);
                std::process::exit(1);
            }
        },

        Commands::Logs { level, follow } => {
            let level_str = level.as_deref();
            match client.get_logs(level_str).await {
                Ok(mut response) => {
                    if *follow {
                        println!("=== Following logs (Ctrl+C to stop) ===");
                    }

                    // For SSE streaming logs in reqwest 0.13.x, read chunks until stream ends
                    let mut buffer = Vec::new();
                    while let Ok(chunk) = response.chunk().await {
                        match chunk {
                            Some(data) => {
                                buffer.clear();
                                buffer.extend_from_slice(&data);
                                let text = String::from_utf8_lossy(&buffer);
                                print!("{}", text);
                                std::io::Write::flush(&mut std::io::stdout())?;
                            }
                            None => {
                                // Stream has ended
                                break;
                            }
                        }

                        if !*follow {
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get logs: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Version => match client.get_version().await {
            Ok(version) => {
                println!("Mihomo version: {}", version);
            }
            Err(e) => {
                eprintln!("Failed to get version: {}", e);
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
