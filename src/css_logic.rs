use cssparser::{
    self, AtRuleParser, CowRcStr, ParseError, Parser, ParserInput, QualifiedRuleParser,
    StyleSheetParser, Token,
};

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

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Px,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Stylesheet {
    pub fn new() -> Stylesheet {
        Stylesheet { rules: Vec::new() }
    }
}

impl SimpleSelector {
    pub fn new(tag_name: Option<String>, id: Option<String>, class: Vec<String>) -> SimpleSelector {
        SimpleSelector {
            tag_name,
            id,
            class,
        }
    }
}

impl Value {
    pub fn px(v: f32) -> Value {
        Value::Length(v, Unit::Px)
    }
}

struct RuleParser;

impl<'i> QualifiedRuleParser<'i> for RuleParser {
    type Prelude = Vec<Selector>;
    type QualifiedRule = Rule;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        let mut selectors = Vec::new();
        loop {
            selectors.push(Selector::Simple(parse_simple_selector(input)?));
            if input.try_parse(|input| input.expect_comma()).is_err() {
                break;
            }
        }
        Ok(selectors)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &cssparser::ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, ParseError<'i, Self::Error>> {
        let mut declarations = Vec::new();
        loop {
            match input.next() {
                Ok(Token::Ident(name)) => {
                    let name = name.to_string();
                    if input.expect_colon().is_ok() {
                        if let Ok(value) = parse_css_value(input) {
                            declarations.push(Declaration { name, value });
                            match input.next() {
                                Ok(Token::Semicolon) => continue,
                                Ok(Token::CloseCurlyBracket) => break,
                                Err(_) => break,
                                _ => {} // Fallthrough to skip
                            }
                        }
                    }
                }
                Ok(Token::CloseCurlyBracket) => break,
                Err(_) => break,
                Ok(Token::Semicolon) => continue,
                _ => {} // Fallthrough to skip
            }

            // Skip until semicolon or end of block
            while let Ok(t) = input.next() {
                match t {
                    Token::Semicolon => break,
                    Token::CloseCurlyBracket => {
                        return Ok(Rule {
                            selector: prelude,
                            declaration: declarations,
                        });
                    }
                    _ => {}
                }
            }
        }
        Ok(Rule {
            selector: prelude,
            declaration: declarations,
        })
    }
}

impl<'i> AtRuleParser<'i> for RuleParser {
    type Prelude = ();
    type AtRule = Rule;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        _name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::Prelude, ParseError<'i, Self::Error>> {
        Err(input.new_custom_error(()))
    }

    fn parse_block<'t>(
        &mut self,
        _prelude: Self::Prelude,
        _start: &cssparser::ParserState,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self::AtRule, ParseError<'i, Self::Error>> {
        Err(input.new_custom_error(()))
    }
}

fn parse_simple_selector<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> Result<SimpleSelector, ParseError<'i, ()>> {
    let mut selector = SimpleSelector {
        tag_name: None,
        id: None,
        class: Vec::new(),
    };
    let mut matched = false;

    if let Ok(name) = input.try_parse(|input| input.expect_ident().map(|s| s.to_string())) {
        selector.tag_name = Some(name);
        matched = true;
    } else if input.try_parse(|input| input.expect_delim('*')).is_ok() {
        matched = true;
    }

    loop {
        let state = input.state();
        match input.next() {
            Ok(Token::Hash(name)) | Ok(Token::IDHash(name)) => {
                selector.id = Some(name.to_string());
                matched = true;
            }
            Ok(Token::Delim('.')) => {
                let name = match input.expect_ident() {
                    Ok(t) => t.to_string(),
                    Err(_) => return Err(input.new_custom_error(())),
                };
                selector.class.push(name);
                matched = true;
            }
            _ => {
                input.reset(&state);
                break;
            }
        }
    }

    if !matched {
        return Err(input.new_custom_error(()));
    }

    Ok(selector)
}

fn parse_css_value<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Value, ParseError<'i, ()>> {
    match input.next() {
        Ok(Token::Dimension { value, unit, .. }) => {
            if unit.eq_ignore_ascii_case("px") {
                Ok(Value::Length(*value, Unit::Px))
            } else {
                Ok(Value::Length(*value, Unit::Px))
            }
        }
        Ok(Token::Number { value, .. }) => Ok(Value::Length(*value, Unit::Px)),
        Ok(Token::Hash(name)) | Ok(Token::IDHash(name)) => {
            let color = parse_color_hex(&name);
            Ok(Value::ColorValue(color))
        }
        Ok(Token::Ident(name)) => Ok(Value::Keyword(name.to_string())),
        _ => Err(input.new_custom_error(())),
    }
}

fn parse_color_hex(hex: &str) -> Color {
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color { r, g, b, a: 255 }
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0) * 17;
        let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0) * 17;
        let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0) * 17;
        Color { r, g, b, a: 255 }
    } else {
        Color { r: 0, g: 0, b: 0, a: 255 }
    }
}

pub fn parse(source: String) -> Stylesheet {
    let mut input = ParserInput::new(&source);
    let mut parser = Parser::new(&mut input);
    let mut rule_parser = RuleParser;
    let mut rule_list_parser = StyleSheetParser::new(&mut parser, &mut rule_parser);

    let mut rules = Vec::new();
    while let Some(rule) = rule_list_parser.next() {
        if let Ok(r) = rule {
            rules.push(r);
        }
    }

    Stylesheet { rules }
}
