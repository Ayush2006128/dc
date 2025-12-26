use std::default::Default;
use std::io::Cursor;
// document parser
use html5ever::parse_document;
// config for parser
use html5ever::driver::ParseOpts;
// trait to feed strings to parser
use html5ever::tendril::TendrilSink;

// premade dom structure types
use markup5ever_rcdom::{Handle, NodeData, RcDom};

pub fn walk(handle: &Handle, indent: usize) {
    let indent_str = " ".repeat(indent);

    // node
    let node = handle;

    match node.data {
        NodeData::Document => {
            println!("{} #Document", indent_str);
        }

        NodeData::Text { ref contents } => {
            let text = contents.borrow();
            let text = text.trim();
            if !text.is_empty() {
                println!("{}#Text: \"{}\"", indent_str, text)
            }
        }
        NodeData::Comment { ref contents } => {
            println!("{}", contents)
        }
        NodeData::Element {
            ref name,
            ref attrs,
            .. 
        } => {
            // 'name.local' gives us the tag name (e.g., "div", "p")
            print!("{}<{}>", indent_str, name.local);

            // Print attributes (e.g., class="foo")
            for attr in attrs.borrow().iter() {
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(); // Finish the line
        }
        _ => {}
    }
    for child in node.children.borrow().iter() {
        walk(child, indent + 2);
    }
}

pub fn parse_html(html_input: String) -> RcDom {
    let dom_skeleton = RcDom::default();

    let mut reader = Cursor::new(html_input.as_bytes());

    let dom = parse_document(dom_skeleton, ParseOpts::default())
        .from_utf8()
        .read_from(&mut reader)
        .expect("Failed to parse html"); // Unwrap allows us to crash if parsing totally fails
    dom
}

// ===Tests===
#[cfg(test)]
mod test_html {
    use super::*;

    #[test]
    fn test_parser() {
        let html = r#" 
   <!DOCTYPE html>
            <html>
                <body>
                    <div id="test">Hello Lib</div>
                </body>
            </html>
        "#;
        let dom = parse_html(html.parse().unwrap());
        println!("--- Test output ---");
        walk(&dom.document, 0);
    }
}
