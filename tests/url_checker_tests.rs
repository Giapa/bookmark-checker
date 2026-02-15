use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use bookmark_checker::html_parser;
use bookmark_checker::url_checker;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Helper: build a bookmark HTML string with the given URLs and parse it into a bookmarks map.
fn bookmarks_from_urls(urls: &[&str]) -> (markup5ever_rcdom::RcDom, HashMap<String, Vec<markup5ever_rcdom::Handle>>) {
    let links: String = urls
        .iter()
        .enumerate()
        .map(|(i, url)| format!("<dt><a href=\"{}\" add_date=\"{}\">Link {}</a>\n", url, 1000000 + i, i))
        .collect();

    let html = format!(
        r#"<!DOCTYPE netscape-bookmark-file-1>
<html><head><title>Bookmarks</title></head><body><h1>Bookmarks</h1>
<dl><p>
{}
</p></dl>
</body></html>"#,
        links
    );

    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp_path = format!("tests/fixtures/url_checker_tmp_{}.html", id);
    std::fs::write(&tmp_path, &html).unwrap();
    let dom = html_parser::parse_html_file(&tmp_path).unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    std::fs::remove_file(&tmp_path).ok();
    (dom, bookmarks)
}

#[tokio::test]
async fn check_urls_detects_404() {
    let server = MockServer::start().await;

    Mock::given(path("/ok"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(path("/not-found"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let ok_url = format!("{}/ok", server.uri());
    let bad_url = format!("{}/not-found", server.uri());
    let urls = vec![ok_url.as_str(), bad_url.as_str()];

    let (_dom, bookmarks) = bookmarks_from_urls(&urls);
    let outdated = url_checker::check_urls(&bookmarks).await;

    assert_eq!(outdated.len(), 1, "Should detect exactly 1 outdated URL");
    assert!(
        outdated.contains_key(&bad_url),
        "The 404 URL should be in the outdated list"
    );
    assert!(
        !outdated.contains_key(&ok_url),
        "The 200 URL should NOT be in the outdated list"
    );
}

#[tokio::test]
async fn check_urls_all_ok_returns_empty() {
    let server = MockServer::start().await;

    Mock::given(path("/page1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    Mock::given(path("/page2"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let url1 = format!("{}/page1", server.uri());
    let url2 = format!("{}/page2", server.uri());
    let urls = vec![url1.as_str(), url2.as_str()];

    let (_dom, bookmarks) = bookmarks_from_urls(&urls);
    let outdated = url_checker::check_urls(&bookmarks).await;

    assert_eq!(outdated.len(), 0, "No URLs should be outdated when all return 200");
}

#[tokio::test]
async fn check_urls_all_404_returns_all() {
    let server = MockServer::start().await;

    Mock::given(path("/dead1"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    Mock::given(path("/dead2"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let url1 = format!("{}/dead1", server.uri());
    let url2 = format!("{}/dead2", server.uri());
    let urls = vec![url1.as_str(), url2.as_str()];

    let (_dom, bookmarks) = bookmarks_from_urls(&urls);
    let outdated = url_checker::check_urls(&bookmarks).await;

    assert_eq!(outdated.len(), 2, "Both 404 URLs should be outdated");
}

#[tokio::test]
async fn check_urls_non_404_errors_are_not_flagged() {
    let server = MockServer::start().await;

    // 500 Internal Server Error — should NOT be treated as outdated
    Mock::given(path("/error"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    // 403 Forbidden — should NOT be treated as outdated
    Mock::given(path("/forbidden"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let url1 = format!("{}/error", server.uri());
    let url2 = format!("{}/forbidden", server.uri());
    let urls = vec![url1.as_str(), url2.as_str()];

    let (_dom, bookmarks) = bookmarks_from_urls(&urls);
    let outdated = url_checker::check_urls(&bookmarks).await;

    assert_eq!(outdated.len(), 0, "Non-404 status codes should not be flagged as outdated");
}

#[tokio::test]
async fn check_urls_empty_bookmarks() {
    let bookmarks: HashMap<String, Vec<markup5ever_rcdom::Handle>> = HashMap::new();
    let outdated = url_checker::check_urls(&bookmarks).await;
    assert_eq!(outdated.len(), 0, "Empty bookmarks should return empty outdated list");
}
