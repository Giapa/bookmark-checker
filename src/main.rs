mod html_parser;
mod url_checker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "./merika-bookmarks.html";
    let dom: markup5ever_rcdom::RcDom = html_parser::parse_html_file(&file_path)?;
    let bookmarks = html_parser::get_all_bookmarks(&dom);
    let duplicate_bookmarks = html_parser::list_duplicate_bookmarks(&bookmarks);
    for (url, nodes) in &duplicate_bookmarks {
        println!("Duplicate URL found: {} (nodes {})", url, nodes.len());
    }
    url_checker::check_urls(&bookmarks);
    Ok(())
}
