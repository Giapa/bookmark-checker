use futures::future::join_all;
use markup5ever_rcdom::Handle;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

pub async fn check_urls(bookmarks: &HashMap<String, Vec<Handle>>) -> HashMap<String, Vec<Handle>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap();

    println!("Checking {} unique URLs concurrently...\n", bookmarks.len());

    let mut tasks = Vec::new();

    for (url, _) in bookmarks {
        let client = client.clone();
        let url = url.clone();

        let task: tokio::task::JoinHandle<Option<String>> = tokio::spawn(async move {
            match client
                .get(&url)
                .header(
                    "Accept",
                    "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
                )
                .header("Accept-Language", "en-US,en;q=0.5")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("DNT", "1")
                .header("Connection", "keep-alive")
                .header("Upgrade-Insecure-Requests", "1")
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if status.as_u16() == 404 {
                        println!("❌ 404 NOT FOUND: {}", url);
                        Some(url)
                    } else {
                        None
                    }
                }
                Err(_e) => None,
            }
        });

        tasks.push(task);
    }

    let results = join_all(tasks).await;

    let mut outdated_urls: HashMap<String, Vec<Handle>> = HashMap::new();
    for result in results {
        if let Ok(Some(url)) = result {
            if let Some(handles) = bookmarks.get(&url) {
                if handles.len() > 1 {
                    println!("   (appears {} times)", handles.len());
                }
                outdated_urls.insert(url, handles.clone());
            }
        }
    }

    println!("\nFound {} outdated URLs", outdated_urls.len());
    outdated_urls
}
