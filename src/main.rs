use bookmark_checker::html_parser;
use bookmark_checker::url_checker;
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "bookmark-checker", about = "Check and clean HTML bookmark files")]
struct Args {
    /// Path to the input HTML bookmarks file
    file_path: PathBuf,

    /// Output file path (defaults to <input>-cleaned.html)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn default_output_path(input: &Path) -> PathBuf {
    let stem = input.file_stem().unwrap_or_default().to_string_lossy();
    let ext = input.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    input.with_file_name(format!("{}-cleaned{}", stem, ext))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file_path = &args.file_path;
    let output_path = args.output.unwrap_or_else(|| default_output_path(file_path));

    let dom: markup5ever_rcdom::RcDom = html_parser::parse_html_file(&file_path.to_string_lossy())?;
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    let duplicate_bookmarks = html_parser::list_duplicate_bookmarks(&bookmarks);
    if !duplicate_bookmarks.is_empty() {
        for (url, nodes) in &duplicate_bookmarks {
            println!("Duplicate URL found: {} (nodes {})", url, nodes.len());
        }
        println!("\nRemoving {} duplicate URLs...", duplicate_bookmarks.len());
        html_parser::remove_duplicate_bookmarks(&duplicate_bookmarks);
        html_parser::save_html_to_file(&dom, &output_path.to_string_lossy())?;
    } else {
        println!("No duplicate URLs found!");
    }

    let outdated_urls = url_checker::check_urls(&bookmarks).await;

    if !outdated_urls.is_empty() {
        println!("\nRemoving {} outdated URLs...", outdated_urls.len());
        html_parser::remove_bookmarks(&outdated_urls);
        html_parser::save_html_to_file(&dom, &output_path.to_string_lossy())?;
    } else {
        println!("\nNo outdated URLs found!");
    }

    println!("\nOutput written to: {}", output_path.display());

    Ok(())
}
