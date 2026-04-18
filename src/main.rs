// File: main.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Command-line interface for Auria Node.
//     Provides CLI commands for starting/stopping the node, checking status,
//     configuring tiers, wallet management, and cluster operations.
//
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use reqwest::Client;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(
    name = "auria-cli",
    version = "1.0.0",
    about = "Auria CLI - Command Line Interface for Auria Node"
)]
struct Cli {
    #[arg(short, long, default_value = "http://localhost:8080")]
    url: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Chat {
        #[arg(short, long)]
        model: Option<String>,
        
        #[arg(short, long, default_value = "100")]
        max_tokens: u32,
        
        message: String,
    },
    Complete {
        #[arg(short, long)]
        model: Option<String>,
        
        prompt: String,
    },
    Models,
    Status,
    Health,
    Metrics,
    Peers {
        #[arg(long)]
        list: bool,
        
        #[arg(long)]
        connect: Option<String>,
        
        #[arg(long)]
        disconnect: Option<String>,
    },
    Settlement {
        #[arg(long)]
        status: bool,
        
        #[arg(long)]
        submit: bool,
        
        #[arg(long)]
        withdraw: bool,
        
        #[arg(long)]
        history: bool,
    },
    Cluster {
        #[arg(long)]
        status: bool,
        
        #[arg(long)]
        workers: bool,
        
        #[arg(long)]
        add_worker: Option<String>,
    },
    Model {
        #[arg(long)]
        status: bool,
        
        #[arg(long)]
        load: Option<String>,
    },
    Wallet {
        #[arg(long)]
        create: bool,
        
        #[arg(long)]
        import: Option<String>,
        
        #[arg(long)]
        address: bool,
        
        #[arg(long)]
        balance: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = Client::new();
    let base_url = cli.url.trim_end_matches('/');

    match cli.cmd {
        Command::Chat { model, max_tokens, message } => {
            let model_name = model.unwrap_or_else(|| "nano".to_string());
            let request = json!({
                "model": model_name,
                "messages": [
                    {"role": "user", "content": message}
                ],
                "max_tokens": max_tokens,
                "stream": false
            });

            let response = client
                .post(format!("{}/v1/chat/completions", base_url))
                .json(&request)
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let content = &data["choices"][0]["message"]["content"];
                println!("{}", content);
            } else {
                eprintln!("Error: {}", response.text().await?);
            }
        }

        Command::Complete { model, prompt } => {
            let model_name = model.unwrap_or_else(|| "nano".to_string());
            let request = json!({
                "model": model_name,
                "prompt": prompt,
                "max_tokens": 100
            });

            let response = client
                .post(format!("{}/v1/completions", base_url))
                .json(&request)
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let text = &data["choices"][0]["text"];
                println!("{}", text);
            } else {
                eprintln!("Error: {}", response.text().await?);
            }
        }

        Command::Models => {
            let response = client
                .get(format!("{}/v1/models", base_url))
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                println!("Available models:");
                for model in data["data"].as_array().unwrap_or(&vec![]) {
                    println!("  - {} (created: {})", 
                        model["id"], 
                        model["created"]
                    );
                }
            } else {
                eprintln!("Error: {}", response.text().await?);
            }
        }

        Command::Status => {
            let response = client
                .get(format!("{}/api/v1/status", base_url))
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                println!("=== Node Status ===");
                println!("Node ID: {}", data["node_id"]);
                println!("P2P Enabled: {}", data["p2p_enabled"]);
                println!("Active Requests: {}", data["active_requests"]);
                println!("Peers: {}", data["peers"].as_array().map(|p| p.len()).unwrap_or(0));
            } else {
                eprintln!("Error: {}", response.text().await?);
            }
        }

        Command::Health => {
            let response = client
                .get(format!("{}/health", base_url))
                .send()
                .await?;

            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                println!("Status: {}", data["status"]);
                println!("Version: {}", data["version"]);
            } else {
                eprintln!("Error: {}", response.text().await?);
            }
        }

        Command::Metrics => {
            let response = client
                .get(format!("{}/metrics", base_url))
                .send()
                .await?;

            if response.status().is_success() {
                println!("{}", response.text().await?);
            } else {
                eprintln!("Error: {}", response.text().await?);
            }
        }

        Command::Peers { list, connect, disconnect } => {
            if list {
                let response = client
                    .get(format!("{}/api/v1/peers", base_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let data: serde_json::Value = response.json().await?;
                    println!("=== Connected Peers ===");
                    println!("Count: {}", data["count"]);
                    for peer in data["peers"].as_array().unwrap_or(&vec![]) {
                        println!("  - {} ({}) [connected: {}]", 
                            peer["node_id"],
                            peer["address"],
                            peer["connected_at"]
                        );
                    }
                }
            }

            if let Some(addr) = connect {
                let request = json!({
                    "address": addr,
                    "port": 9000
                });

                let response = client
                    .post(format!("{}/api/v1/peers/connect", base_url))
                    .json(&request)
                    .send()
                    .await?;

                let data: serde_json::Value = response.json().await?;
                println!("Connect: {} - {}", data["success"], data["message"]);
            }

            if let Some(addr) = disconnect {
                let request = json!({
                    "address": addr,
                    "port": 9000
                });

                let response = client
                    .post(format!("{}/api/v1/peers/disconnect", base_url))
                    .json(&request)
                    .send()
                    .await?;

                let data: serde_json::Value = response.json().await?;
                println!("Disconnect: {} - {}", data["success"], data["message"]);
            }
        }

        Command::Settlement { status, submit, withdraw, history } => {
            if status {
                let response = client
                    .get(format!("{}/api/v1/settlement/status", base_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let data: serde_json::Value = response.json().await?;
                    println!("=== Settlement Status ===");
                    println!("Connected: {}", data["connected"]);
                    if data["connected"].as_bool().unwrap_or(false) {
                        println!("Wallet: {}", data["wallet_address"]);
                        println!("Chain ID: {}", data["chain_id"]);
                        println!("Pending Receipts: {}", data["pending_receipts"]);
                        println!("Total Settled: {}", data["total_settled"]);
                        println!("Pending Rewards: {}", data["pending_rewards"]);
                    }
                }
            }

            if submit {
                let response = client
                    .post(format!("{}/api/v1/settlement/submit", base_url))
                    .send()
                    .await?;

                let data: serde_json::Value = response.json().await?;
                println!("Submit Settlement: {} - {}", data["success"], data["message"]);
                if data["success"].as_bool().unwrap_or(false) {
                    println!("TX Hash: {}", data["tx_hash"].as_str().unwrap_or("N/A"));
                }
            }

            if withdraw {
                let response = client
                    .post(format!("{}/api/v1/settlement/withdraw", base_url))
                    .send()
                    .await?;

                let data: serde_json::Value = response.json().await?;
                println!("Withdraw: {} - {}", data["success"], data["message"]);
            }

            if history {
                let response = client
                    .get(format!("{}/api/v1/settlement/history", base_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let data: serde_json::Value = response.json().await?;
                    println!("=== Settlement History ===");
                    println!("Total: {}", data["total"]);
                    for sub in data["submissions"].as_array().unwrap_or(&vec![]) {
                        println!("  [{}] {} receipts - {} (gas: {})", 
                            sub["status"],
                            sub["receipt_count"],
                            sub["tx_hash"],
                            sub["gas_used"].as_u64().unwrap_or(0)
                        );
                    }
                }
            }
        }

        Command::Cluster { status, workers, add_worker } => {
            if status {
                let response = client
                    .get(format!("{}/api/v1/cluster/status", base_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let data: serde_json::Value = response.json().await?;
                    println!("=== Cluster Status ===");
                    println!("Node ID: {}", data["node_id"]);
                    println!("Is Leader: {}", data["is_leader"]);
                    println!("Leader ID: {}", data["leader_id"].as_str().unwrap_or("N/A"));
                    println!("Workers: {}", data["total_workers"]);
                    println!("Pending Tasks: {}", data["pending_tasks"]);
                    
                    if let Some(raft) = data["raft_info"].as_object() {
                        println!("Raft Role: {}", raft["role"]);
                        println!("Raft Term: {}", raft["term"]);
                    }
                }
            }

            if workers {
                let response = client
                    .get(format!("{}/api/v1/cluster/workers", base_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let data: serde_json::Value = response.json().await?;
                    println!("=== Cluster Workers ===");
                    println!("Total: {}", data["total_workers"]);
                    println!("Idle: {}", data["idle_workers"]);
                    println!("Busy: {}", data["busy_workers"]);
                    println!("Offline: {}", data["offline_workers"]);
                }
            }

            if let Some(addr) = add_worker {
                let request = json!({
                    "id": format!("worker-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
                    "address": addr,
                    "capabilities": "standard",
                    "memory_total_mb": 8192,
                    "cpu_cores": 4,
                    "gpu_available": false
                });

                let response = client
                    .post(format!("{}/api/v1/cluster/workers/add", base_url))
                    .json(&request)
                    .send()
                    .await?;

                let data: serde_json::Value = response.json().await?;
                println!("Add Worker: {} - {}", data["success"], data["message"]);
            }
        }

        Command::Model { status, load } => {
            if status {
                let response = client
                    .get(format!("{}/api/v1/model/status", base_url))
                    .send()
                    .await?;

                if response.status().is_success() {
                    let data: serde_json::Value = response.json().await?;
                    println!("=== Model Status ===");
                    println!("Loaded: {}", data["loaded"]);
                    if data["loaded"].as_bool().unwrap_or(false) {
                        println!("Model Path: {}", data["model_path"].as_str().unwrap_or("N/A"));
                    }
                }
            }

            if let Some(path) = load {
                let request = json!({
                    "model_path": path
                });

                let response = client
                    .post(format!("{}/api/v1/model/load", base_url))
                    .json(&request)
                    .send()
                    .await?;

                let data: serde_json::Value = response.json().await?;
                println!("Load Model: {} - {}", data["success"], data["message"]);
            }
        }

        Command::Wallet { create, import, address, balance } => {
            if create {
                println!("Creating new wallet...");
                println!("(Wallet creation would be handled by the node)");
            }

            if let Some(_mnemonic) = import {
                println!("Importing wallet from mnemonic...");
            }

            if address {
                println!("Getting wallet address...");
            }

            if balance {
                println!("Getting wallet balance...");
            }
        }
    }

    Ok(())
}
