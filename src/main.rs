use dc::{css_logic, parse_html, walk};

fn main() {
    println!("Showing html ");
    println!();
    let input = r#"
   <!DOCTYPE html>
            <html>
                <body>
                    <div id="test">Hello Lib</div>
                </body>
            </html>
        "#;

    let parsed_html = parse_html(input.parse().unwrap());
    walk(&parsed_html.document, 0);
    println!();
    println!("showing css");
    println!();
    let css_input = "
        h1, div {
            display: block;
            margin: 10px;
        }
        #answer {
            color: #FF0000;
        }
    ";

    let stylesheet = css_logic::parse(css_input.to_string());

    println!("{:#?}", stylesheet);
}
