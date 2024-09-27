mod config;
mod notify;
use std::{process::Command, thread::sleep, time::Duration};

use regex::Regex;
use tokio::{sync::watch, task::JoinSet};

use tracing::Level;
use tracing_unwrap::ResultExt;

#[tokio::main]
async fn main() {
    // Init tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_level(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect_or_log("Init tracing failed");

    let config = match config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            return;
        }
    };

    // Check config
    if config.least_needed_gpu.is_none() {
        tracing::error!("Missing least_needed_gpu in config");
        return;
    }

    let mut threads = JoinSet::new();
    let (sender, mut reciver) = watch::channel(false);
    threads.spawn(async move {
        while reciver.changed().await.is_ok() {
            if reciver.borrow().clone() {
                if let Some(bark_urls) = &config.bark_urls {
                    notify::notify_bark(bark_urls.clone(), "Enough GPU available".to_string()).await;
                    break;
                } else {
                    tracing::info!("No bark clients to inform");
                }
                break;
            }
        }
        tracing::info!("Done notifying bark");
    });

    let mut reciver = sender.subscribe();
    threads.spawn(async move {
        while reciver.changed().await.is_ok() {
            if reciver.borrow().clone() {
                if let Some(ntfy_urls) = &config.ntfy_urls {
                    notify::notify_ntfy(ntfy_urls.clone(), "Enough GPU available".to_string())
                        .await;
                } else {
                    tracing::info!("No ntfy clients to inform");
                }
                break;
            }   
        }
        tracing::info!("Done notifying ntfy");
    });

    let re = Regex::new(r"(\d+)").expect_or_log("Failed to compile regex");
    loop {
        let gpu_status = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.used,memory.free,memory.total")
            .arg("--format=csv,noheader")
            .output()
            .expect_or_log("Failed to run nvidia-smi");
        let gpu_status =
            String::from_utf8(gpu_status.stdout).expect_or_log("Failed to parse gpu status");
        let mut free_count = 0;
        gpu_status.lines().for_each(|line| {
            let mut numbers = Vec::new();
            for cap in re.captures_iter(line) {
                let num: i32 = cap[1].parse().expect_or_log("Failed to parse stats number");
                numbers.push(num);
            }
            let used = numbers[0];
            tracing::info!(
                "Used: {}, Free: {}, Total: {}",
                numbers[0],
                numbers[1],
                numbers[2]
            );

            if used < 1024 {
                free_count += 1;
            }
        });
        if free_count >= config.least_needed_gpu.unwrap() {
            match sender.send(true) {
                Ok(_) => {
                    tracing::info!("Enough GPU available, notifying");
                }
                Err(e) => {
                    tracing::error!("Failed to notify: {}", e);
                }
            }

            break;
        } else {
            tracing::info!(
                "Not enough GPU available, currently have {} available, waiting for 30s",
                free_count
            );
        }
        sleep(Duration::from_secs(30));
    }

    while threads.join_next().await.is_some() {}
}
