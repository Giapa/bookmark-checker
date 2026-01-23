use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn parse_html_file(file_path: &str) -> Result<RcDom, std::io::Error> {
    let path = Path::new(file_path);
    let contents = fs::read_to_string(path)?;
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut contents.as_bytes())?;
    return Ok(dom);
}

pub fn get_all_bookmarks(dom: &RcDom) -> HashMap<String, Vec<Handle>> {
    let mut bookmarks: HashMap<String, Vec<Handle>> = HashMap::new();
    extract_bookmarks(&dom.document, &mut bookmarks);
    return bookmarks;
}

fn extract_bookmarks(node: &Handle, bookmarks: &mut HashMap<String, Vec<Handle>>) {
    match node.data {
        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            if name.local.as_ref() == "a" {
                for attr in attrs.borrow().iter() {
                    let url = attr.value.to_string();
                    if attr.name.local.as_ref() == "href" && url.contains("http") {
                        bookmarks
                            .entry(url)
                            .or_insert_with(Vec::new)
                            .push(node.clone());
                    }
                }
            }
        }
        _ => {}
    }

    for child in node.children.borrow().iter() {
        extract_bookmarks(child, bookmarks);
    }
}

pub fn list_duplicate_bookmarks(
    bookmarks: &HashMap<String, Vec<Handle>>,
) -> HashMap<String, Vec<Handle>> {
    bookmarks
        .iter()
        .filter(|(_, handles)| handles.len() > 1)
        .map(|(url, handles)| (url.clone(), handles.clone()))
        .collect()
}
