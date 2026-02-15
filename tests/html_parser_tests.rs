use std::fs;
use std::path::Path;

// We need to import from the crate
use bookmark_checker::html_parser;

#[test]
fn parse_html_file_with_valid_file() {
    let result = html_parser::parse_html_file("tests/fixtures/basic.html");
    assert!(result.is_ok(), "Should successfully parse a valid HTML file");
}

#[test]
fn parse_html_file_with_nonexistent_file() {
    let result = html_parser::parse_html_file("tests/fixtures/nonexistent.html");
    assert!(result.is_err(), "Should return error for nonexistent file");
}

#[test]
fn get_all_bookmarks_returns_correct_count() {
    let dom = html_parser::parse_html_file("tests/fixtures/basic.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    assert_eq!(bookmarks.len(), 3, "Should find 3 unique bookmarks");
}

#[test]
fn get_all_bookmarks_contains_expected_urls() {
    let dom = html_parser::parse_html_file("tests/fixtures/basic.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    assert!(bookmarks.contains_key("https://www.example.com"));
    assert!(bookmarks.contains_key("https://www.rust-lang.org"));
    assert!(bookmarks.contains_key("https://github.com"));
}

#[test]
fn get_all_bookmarks_empty_file() {
    let dom = html_parser::parse_html_file("tests/fixtures/empty.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    assert_eq!(bookmarks.len(), 0, "Should find 0 bookmarks in empty file");
}

#[test]
fn list_duplicate_bookmarks_finds_duplicates() {
    let dom = html_parser::parse_html_file("tests/fixtures/duplicates.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    let duplicates = html_parser::list_duplicate_bookmarks(&bookmarks);
    assert_eq!(duplicates.len(), 2, "Should find 2 duplicate URLs");
    assert!(duplicates.contains_key("https://www.example.com"));
    assert!(duplicates.contains_key("https://www.rust-lang.org"));
}

#[test]
fn list_duplicate_bookmarks_none_when_no_duplicates() {
    let dom = html_parser::parse_html_file("tests/fixtures/basic.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    let duplicates = html_parser::list_duplicate_bookmarks(&bookmarks);
    assert_eq!(duplicates.len(), 0, "Should find 0 duplicates");
}

#[test]
fn remove_duplicate_bookmarks_reduces_count() {
    let dom = html_parser::parse_html_file("tests/fixtures/duplicates.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    let duplicates = html_parser::list_duplicate_bookmarks(&bookmarks);

    assert_eq!(duplicates.len(), 2);
    html_parser::remove_duplicate_bookmarks(&duplicates);

    // Re-extract bookmarks after removal
    let bookmarks_after = html_parser::get_all_bookmarks(&dom);
    // Each duplicate URL should still exist (one copy kept), but with only 1 handle
    for (url, handles) in &bookmarks_after {
        assert_eq!(
            handles.len(),
            1,
            "URL {} should have exactly 1 entry after dedup",
            url
        );
    }
}

#[test]
fn remove_bookmarks_removes_all_instances() {
    let dom = html_parser::parse_html_file("tests/fixtures/duplicates.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);

    // Pick one URL to remove entirely
    let mut to_remove = std::collections::HashMap::new();
    let url = "https://www.example.com".to_string();
    if let Some(handles) = bookmarks.get(&url) {
        to_remove.insert(url.clone(), handles.clone());
    }

    html_parser::remove_bookmarks(&to_remove);

    let bookmarks_after = html_parser::get_all_bookmarks(&dom);
    assert!(
        !bookmarks_after.contains_key("https://www.example.com"),
        "Removed URL should no longer be present"
    );
}

#[test]
fn save_html_to_file_creates_output() {
    let dom = html_parser::parse_html_file("tests/fixtures/basic.html").unwrap();
    let output_path = "tests/fixtures/test_output.html";

    let result = html_parser::save_html_to_file(&dom, output_path);
    assert!(result.is_ok(), "Should save file without error");
    assert!(Path::new(output_path).exists(), "Output file should exist");

    // Verify content is non-empty
    let content = fs::read_to_string(output_path).unwrap();
    assert!(!content.is_empty(), "Output file should not be empty");
    assert!(
        content.contains("example.com"),
        "Output should contain bookmark URLs"
    );

    // Clean up
    fs::remove_file(output_path).ok();
}

#[test]
fn remove_duplicates_end_to_end_saved_file_has_no_duplicates() {
    let dom = html_parser::parse_html_file("tests/fixtures/duplicates.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    let duplicates = html_parser::list_duplicate_bookmarks(&bookmarks);

    assert_eq!(duplicates.len(), 2);
    html_parser::remove_duplicate_bookmarks(&duplicates);

    let output_path = "tests/fixtures/dedup_output.html";
    html_parser::save_html_to_file(&dom, output_path).unwrap();

    // Re-parse the saved file from disk and verify no duplicates remain
    let dom2 = html_parser::parse_html_file(output_path).unwrap();
    let bookmarks2 = html_parser::get_all_bookmarks(&dom2);
    let duplicates2 = html_parser::list_duplicate_bookmarks(&bookmarks2);

    assert_eq!(duplicates2.len(), 0, "Saved file should have no duplicates");
    assert_eq!(bookmarks2.len(), 3, "Should still have 3 unique bookmarks");

    // Clean up
    fs::remove_file(output_path).ok();
}

#[test]
fn remove_bookmarks_end_to_end_saved_file_missing_removed_urls() {
    let dom = html_parser::parse_html_file("tests/fixtures/duplicates.html").unwrap();
    let bookmarks = html_parser::get_all_bookmarks(&dom);

    // Remove all instances of example.com
    let mut to_remove = std::collections::HashMap::new();
    let url = "https://www.example.com".to_string();
    if let Some(handles) = bookmarks.get(&url) {
        to_remove.insert(url.clone(), handles.clone());
    }
    html_parser::remove_bookmarks(&to_remove);

    let output_path = "tests/fixtures/removed_output.html";
    html_parser::save_html_to_file(&dom, output_path).unwrap();

    // Re-parse saved file and verify the URL is gone
    let dom2 = html_parser::parse_html_file(output_path).unwrap();
    let bookmarks2 = html_parser::get_all_bookmarks(&dom2);

    assert!(
        !bookmarks2.contains_key("https://www.example.com"),
        "Removed URL should not be in saved file"
    );
    assert_eq!(bookmarks2.len(), 2, "Should have 2 remaining bookmarks");

    // Clean up
    fs::remove_file(output_path).ok();
}
