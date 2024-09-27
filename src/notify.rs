pub async fn notify_bark(bark_urls: Vec<String>, message: String) {
    for url in bark_urls {
        match reqwest::get(url.clone() + message.as_str()).await {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!("Notified {} successful", url);
                } else {
                    tracing::error!(
                        "Failed to notify {} response code {}",
                        url,
                        response.status()
                    );
                }
            }
            Err(e) => {
                tracing::error!("Failed to notify {} {}", url, e);
            }
        }
    }
}

pub async fn notify_ntfy(ntfy_urls: Vec<String>, message: String) {
    let client = reqwest::Client::new();
    for url in ntfy_urls {
        match client.post(url.clone()).body(message.clone()).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!("Notified {} successful", url);
                } else {
                    tracing::error!(
                        "Failed to notify {} response code {}",
                        url,
                        response.status()
                    );
                }
            }
            Err(e) => {
                tracing::error!("Failed to notify {} {}", url, e);
            }
        }
    }
}
