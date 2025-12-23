use std::io;
use dc::{parse_html, walk};

fn main() {
    println!("Enter HTML Code below:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to get input");
    let parsed_html = parse_html(input);
    walk(&parsed_html.document, 0);
}
