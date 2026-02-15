use html5ever::parse_document;
use html5ever::serialize::{SerializeOpts, serialize};
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
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

pub fn remove_duplicate_bookmarks(duplicate_bookmarks: &HashMap<String, Vec<Handle>>) {
    for (url, handles) in duplicate_bookmarks {
        if let Some(handle) = handles.last() {
            println!("Removing last duplicate of: {}", url);
            if let Some(parent) = handle.parent.take().and_then(|weak| weak.upgrade()) {
                parent
                    .children
                    .borrow_mut()
                    .retain(|child| !std::ptr::eq(child as *const _, handle as *const _));
            }
        }
    }
}

pub fn remove_bookmarks(outdated_urls: &HashMap<String, Vec<Handle>>) {
    for (url, handles) in outdated_urls {
        println!("Removing {} instances of: {}", handles.len(), url);
        for handle in handles {
            // Remove the node from its parent
            if let Some(parent) = handle.parent.take().and_then(|weak| weak.upgrade()) {
                parent
                    .children
                    .borrow_mut()
                    .retain(|child| !std::ptr::eq(child as *const _, handle as *const _));
            }
        }
    }
}

pub fn save_html_to_file(dom: &RcDom, file_path: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;
    let mut bytes = Vec::new();

    let serializable = SerializableHandle::from(dom.document.clone());
    serialize(&mut bytes, &serializable, SerializeOpts::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    file.write_all(&bytes)?;
    println!("\nSaved cleaned bookmarks to: {}", file_path);
    Ok(())
}
