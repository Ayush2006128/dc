#[derive(Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub selector: Vec<Selector>,
    pub declaration: Vec<Declaration>,
}

#[derive(Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

// 4. A Declaration is a property name and a value (e.g., "color": "red")
#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

// 5. CSS Values. To keep it simple, we support Keywords (like "block")
// and Lengths (like "10px").
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color), // Renamed to avoid confusion
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
    // We can add Em, Rem, % later
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}


// ===Implementing structs===
impl Stylesheet {
    pub fn new() -> Stylesheet {
        Stylesheet { rules: Vec::new() }
    }
}