use markup5ever_rcdom::Handle;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::time::Duration;

pub fn check_urls(bookmarks: &HashMap<String, Vec<Handle>>) {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .unwrap();

    println!("Checking {} unique URLs...\n", bookmarks.len());

    let mut dead_count = 0;
    let mut ok_count = 0;
    let mut error_count = 0;

    for (url, handles) in bookmarks {
        match client
            .get(url)
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
        {
            Ok(response) => {
                let status = response.status();

                if status.as_u16() == 404 {
                    println!("❌ 404 NOT FOUND: {}", url);
                    if handles.len() > 1 {
                        println!("   (appears {} times)", handles.len());
                    }
                    dead_count += 1;
                } else if status.is_success() {
                    ok_count += 1;
                } else {
                    error_count += 1;
                }
            }
            Err(_e) => {
                error_count += 1;
            }
        }
    }

    println!("\n=== Summary ===");
    println!("✅ OK: {}", ok_count);
    println!("❌ Dead (404): {}", dead_count);
    println!("⚠️  Errors: {}", error_count);
    println!("Total checked: {}", bookmarks.len());
}
