use std::default::Default;
// document parser
use html5ever::parse_document;
// config for parser
use html5ever::driver::ParseOpts;
// trait to feed strings to parser
use html5ever::tendril::TendrilSink;

// premade dom structure types
use markup5ever_rcdom::{NodeData, RcDom, Handle};

pub fn walk(handle: &Handle, indent: usize){
    let indent_str = " ".repeat(indent);

    // node
    let node = handle;

    match node.data {
        NodeData::Document => {
            println!("{} #Document", indent_str);
        }

        NodeData::Text {ref contents} => {
            let text = contents.borrow();
            let text = text.trim();
            if !text.is_empty() {
                println!("{}#Text: \"{}\"", indent_str, text)
            }
        }
        NodeData::Comment {ref contents} => {
            println!("{}", contents)
        }
        NodeData::Element { ref name, ref attrs, .. } => {
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
    // 2. Initialize the DOM (RcDom)
    // We start with a default empty DOM structure
    let dom_skeleton = RcDom::default();

    // 3. Run the Parser
    // parse_document takes the skeleton and options
    // .from_utf8() tells it we are sending string data
    // .read_from() actually does the work
    let dom = parse_document(dom_skeleton, ParseOpts::default())
        .from_utf8()
        .read_from(&mut html_input.as_bytes())
        .unwrap(); // Unwrap allows us to crash if parsing totally fails
    dom
}






