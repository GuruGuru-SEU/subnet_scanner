// src/main.rs

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Table};
use indicatif::{ProgressBar, ProgressStyle};
use ipnet::IpNet;
use rayon::prelude::*;
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::{self, JoinSet};

// --- Structs for Data Handling ---

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GeoLocationResponse {
    status: String,
    country: Option<String>,
    city: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProxyResult {
    #[serde(rename = "IP Address")]
    ip_address: IpAddr,
    #[serde(rename = "Response Time (ms)")]
    response_time_ms: u128,
    #[serde(rename = "Location")]
    location: String,
}

#[derive(Debug, Deserialize)]
struct ProxyInputRecord {
    #[serde(rename = "IP Address")]
    ip_address: String,
}

// --- Command-Line Interface Definition ---

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    source: Source,

    /// The port to scan or test for
    #[arg(short, long, default_value_t = 7890)]
    port: u16,

    /// Initial connection timeout for port scanning in milliseconds
    #[arg(long, default_value_t = 200)]
    scan_timeout: u64,

    /// Timeout for the proxy test in seconds
    #[arg(long, default_value_t = 10)]
    test_timeout: u64,

    /// Print detailed real-time logs.
    #[arg(long, short)]
    verbose: bool,

    /// Save the final results to a specified CSV file
    #[arg(long, short, value_name = "FILE_PATH")]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = false)]
struct Source {
    /// The subnet to scan in CIDR notation (e.g., 192.168.1.0/24)
    #[arg(long)]
    subnet: Option<String>,

    /// Read IP addresses from a CSV file to test (skips scanning)
    #[arg(long, short, value_name = "FILE_PATH")]
    input: Option<PathBuf>,
}

// --- Main Application Logic ---

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // --- Setup UI (Progress Bar for file input, Spinner for subnet scan) ---
    let progress_bar = setup_ui(&cli)?;

    // --- Setup Communication Channel ---
    let (tx, mut rx) = mpsc::channel::<SocketAddr>(200);

    // --- Start Producer Task (Scanner or File Reader) ---
    let producer_cli = cli.clone();
    task::spawn_blocking(move || {
        if let Some(subnet) = producer_cli.source.subnet {
            scan_and_send(subnet, producer_cli.port, producer_cli.scan_timeout, tx);
        } else if let Some(path) = producer_cli.source.input {
            let _ = read_and_send(path, producer_cli.port, tx);
        }
    });

    // --- Main Concurrency Loop (Consumer) ---
    let mut test_tasks = JoinSet::new();
    let mut successful_proxies = Vec::new();

    loop {
        tokio::select! {
            Some(addr) = rx.recv() => {
                log_verbose(&progress_bar, &cli, format!("[{}]   Potential proxy at {}", "FOUND".cyan().bold(), addr));
                test_tasks.spawn(test_proxy(addr, cli.test_timeout));
            },
            Some(res) = test_tasks.join_next(), if !test_tasks.is_empty() => {
                // Only increment progress bar if it's not a spinner
                if progress_bar.length().is_some() { progress_bar.inc(1); }

                match res {
                    Ok(Ok(result)) => { // Task succeeded, and proxy test succeeded
                        log_verbose(&progress_bar, &cli, format!("[{}] {} connected in {}ms", "SUCCESS".green().bold(), result.ip_address, result.response_time_ms));
                        log_verbose(&progress_bar, &cli, format!("[{}]      {} located in {}", "GEO".blue().bold(), result.ip_address, result.location));
                        successful_proxies.push(result);
                    }
                    Ok(Err((addr, e))) => { // Task succeeded, but proxy test failed
                        log_verbose(&progress_bar, &cli, format!("[{}]     {}: {}", "FAIL".red().bold(), addr, e));
                    }
                    Err(e) => { // Task itself failed to execute
                         log_verbose(&progress_bar, &cli, format!("[{}]   A test task failed: {}", "ERROR".yellow().bold(), e));
                    }
                }
            },
            else => break,
        }
    }

    progress_bar.finish_with_message("All tasks completed!");

    // --- Display and Save Results ---
    if successful_proxies.is_empty() {
        println!("\nNo working HTTP proxies were found.");
    } else {
        println!("\n--- Final Results ---");
        successful_proxies.sort_by(|a, b| a.response_time_ms.cmp(&b.response_time_ms));
        display_results(&successful_proxies);

        if let Some(path) = cli.output {
            save_to_csv(&path, &successful_proxies)?;
            println!("\nResults saved to {}", path.display());
        }
    }

    Ok(())
}

// --- Helper and Worker Functions ---

fn setup_ui(cli: &Cli) -> Result<ProgressBar> {
    if let Some(path) = &cli.source.input {
        // Use a progress bar for file input
        let file = std::fs::File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let total_tasks = rdr.records().count() as u64;
        let pb = ProgressBar::new(total_tasks);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")?.progress_chars("##-"));
        Ok(pb)
    } else {
        // Use a spinner for subnet scanning
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg}")?);
        pb.set_message("Scanning subnet...");
        Ok(pb)
    }
}

fn log_verbose(pb: &ProgressBar, cli: &Cli, msg: String) {
    if cli.verbose {
        pb.println(msg);
    }
}

fn scan_and_send(subnet_str: String, port: u16, timeout_ms: u64, tx: mpsc::Sender<SocketAddr>) {
    if let Ok(network) = subnet_str.parse::<IpNet>() {
        let hosts_to_scan: Vec<IpAddr> = network.hosts().collect();
        hosts_to_scan.into_par_iter().for_each(|ip| {
            let addr = SocketAddr::new(ip, port);
            let timeout = Duration::from_millis(timeout_ms);
            if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                let _ = tx.blocking_send(addr);
            }
        });
    }
}

fn read_and_send(path: PathBuf, default_port: u16, tx: mpsc::Sender<SocketAddr>) -> Result<()> {
    let file = std::fs::File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: ProxyInputRecord = result?;
        // Handle both IP:PORT and just IP formats from input CSV
        if let Ok(addr) = record.ip_address.parse::<SocketAddr>() {
            let _ = tx.blocking_send(addr);
        } else if let Ok(ip) = record.ip_address.parse::<IpAddr>() {
            let addr = SocketAddr::new(ip, default_port);
            let _ = tx.blocking_send(addr);
        }
    }
    Ok(())
}

async fn test_proxy(addr: SocketAddr, timeout_sec: u64) -> Result<ProxyResult, (SocketAddr, anyhow::Error)> {
    let test_logic = async {
        const GEO_API_URL: &str = "http://ip-api.com/json";
        let timeout = Duration::from_secs(timeout_sec);
        let proxy_addr_str = format!("http://{}", addr);
        let proxy = Proxy::all(proxy_addr_str)?;
        let client = reqwest::Client::builder().proxy(proxy).timeout(timeout).build()?;

        let start_time = Instant::now();
        let response = client.get(GEO_API_URL).send().await?;
        let response_time = start_time.elapsed();

        let geo_info = response.json::<GeoLocationResponse>().await?;

        if geo_info.status == "success" {
            let city = geo_info.city.unwrap_or_else(|| "Unknown".to_string());
            let country = geo_info.country.unwrap_or_else(|| "Unknown".to_string());
            Ok(ProxyResult {
                ip_address: addr.ip(), // Store only the IP address
                response_time_ms: response_time.as_millis(),
                location: format!("{}, {}", city, country),
            })
        } else {
            let err_msg = geo_info.message.unwrap_or_else(|| "API error".to_string());
            Err(anyhow::anyhow!("Geo API error: {}", err_msg))
        }
    };
    test_logic.await.map_err(|e| (addr, e))
}

fn display_results(results: &[ProxyResult]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL).set_header(vec![
        "Rank",
        "IP Address",
        "Response Time",
        "Location",
    ]);

    for (i, result) in results.iter().enumerate() {
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(result.ip_address.to_string()),
            Cell::new(format!("{} ms", result.response_time_ms)),
            Cell::new(&result.location),
        ]);
    }

    println!("{table}");
}

fn save_to_csv(path: &PathBuf, results: &[ProxyResult]) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)?;
    for result in results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;
    Ok(())
}