use crate::css_logic::{Rule, Selector, SimpleSelector, Stylesheet, Value};
use std::collections::HashMap;
use markup5ever_rcdom::{Handle, NodeData};

#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: Handle,
    pub specified_values: HashMap<String, Value>,
    pub children: Vec<StyledNode<'a>>,
}

pub fn matches(elem: &Handle, selector: &SimpleSelector) -> bool {
    if let NodeData::Element {
        ref name,
        ref attrs,
        ..
    } = elem.data
    {
        // Check tag name
        if let Some(tag_name) = &selector.tag_name {
            if &name.local.to_string() != tag_name {
                return false;
            }
        }

        let attrs_map: HashMap<String, String> = attrs
            .borrow()
            .iter()
            .map(|attr| (attr.name.local.to_string(), attr.value.to_string()))
            .collect();

        // Check ID
        if let Some(id) = &selector.id {
            if attrs_map.get("id") != Some(id) {
                return false;
            }
        }

        // Check classes
        for class_name in &selector.class {
            if let Some(class_attr) = attrs_map.get("class") {
                if !class_attr.split_whitespace().any(|c| c == class_name) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    } else {
        false
    }
}

type MatchedRule<'a> = (usize, &'a Rule);

fn match_selector<'a>(elem: &Handle, stylesheet: &'a Stylesheet) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .enumerate()
        .filter_map(|(i, rule)| {
            // For now, assume only one simple selector per rule
            rule.selector
                .iter()
                .find(|s| match s {
                    Selector::Simple(simple_s) => matches(elem, simple_s),
                })
                .map(|_| (i, rule))
        })
        .collect()
}

pub fn get_specified_values<'a>(elem: &Handle, stylesheet: &'a Stylesheet) -> HashMap<String, Value> {
    let mut values = HashMap::new();
    let mut matched_rules = match_selector(elem, stylesheet);

    // Sort by specificity (higher index means higher specificity for now)
    matched_rules.sort_by(|(i1, _), (i2, _)| i1.cmp(i2));

    for (_, rule) in matched_rules {
        for declaration in &rule.declaration {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    values
}

pub fn style_tree<'a>(root: &Handle, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    let mut styled_node = StyledNode {
        node: root.clone(),
        specified_values: HashMap::new(),
        children: Vec::new(),
    };

    if let NodeData::Element { .. } = root.data {
        styled_node.specified_values = get_specified_values(root, stylesheet);
    }

    for child in root.children.borrow().iter() {
        styled_node.children.push(style_tree(child, stylesheet));
    }

    styled_node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css_logic;
    use crate::html_logic::parse_html;

    fn get_elem_by_tag(root: &Handle, tag_name: &str) -> Option<Handle> {
        if let NodeData::Element { ref name, .. } = root.data {
            if name.local.as_ref() == tag_name {
                return Some(root.clone());
            }
        }
        for child in root.children.borrow().iter() {
            if let Some(node) = get_elem_by_tag(child, tag_name) {
                return Some(node);
            }
        }
        None
    }

    #[test]
    fn test_matches_tag() {
        let html = r#"<div></div>"#;
        let dom = parse_html(html.to_string());
        let div_node = get_elem_by_tag(&dom.document, "div").expect("div not found");

        let selector = SimpleSelector::new(Some("div".to_string()), None, Vec::new());
        assert!(matches(&div_node, &selector));

        let selector = SimpleSelector::new(Some("p".to_string()), None, Vec::new());
        assert!(!matches(&div_node, &selector));
    }

    #[test]
    fn test_matches_id() {
        let html = r#"<div id="test-id"></div>"#;
        let dom = parse_html(html.to_string());
        let div_node = get_elem_by_tag(&dom.document, "div").expect("div not found");

        let selector = SimpleSelector::new(None, Some("test-id".to_string()), Vec::new());
        assert!(matches(&div_node, &selector));

        let selector = SimpleSelector::new(None, Some("wrong-id".to_string()), Vec::new());
        assert!(!matches(&div_node, &selector));
    }

    #[test]
    fn test_matches_class() {
        let html = r#"<div class="test-class other-class"></div>"#;
        let dom = parse_html(html.to_string());
        let div_node = get_elem_by_tag(&dom.document, "div").expect("div not found");

        let selector = SimpleSelector::new(None, None, vec!["test-class".to_string()]);
        assert!(matches(&div_node, &selector));

        let selector = SimpleSelector::new(None, None, vec!["other-class".to_string()]);
        assert!(matches(&div_node, &selector));

        let selector = SimpleSelector::new(None, None, vec!["wrong-class".to_string()]);
        assert!(!matches(&div_node, &selector));

        let selector = SimpleSelector::new(
            None,
            None,
            vec!["test-class".to_string(), "other-class".to_string()],
        );
        assert!(matches(&div_node, &selector));
    }

    #[test]
    fn test_matches_tag_and_id() {
        let html = r#"<div id="test-id"></div>"#;
        let dom = parse_html(html.to_string());
        let div_node = get_elem_by_tag(&dom.document, "div").expect("div not found");

        let selector = SimpleSelector::new(Some("div".to_string()), Some("test-id".to_string()), Vec::new());
        assert!(matches(&div_node, &selector));

        let selector = SimpleSelector::new(Some("p".to_string()), Some("test-id".to_string()), Vec::new());
        assert!(!matches(&div_node, &selector));
    }

    #[test]
    fn test_get_specified_values() {
        let html = r#"<div id="test-div" class="my-class">Hello</div>"#;
        let css = r#"
            div {
                display: block;
                margin: 10px;
            }
            #test-div {
                color: #FF0000;
            }
            .my-class {
                font-size: 12px;
            }
        "#;
        let dom = parse_html(html.to_string());
        let stylesheet = css_logic::parse(css.to_string());

        let div_node = get_elem_by_tag(&dom.document, "div").expect("div not found");
        let values = get_specified_values(&div_node, &stylesheet);

        assert_eq!(values.get("display"), Some(&Value::Keyword("block".to_string())));
        assert_eq!(values.get("margin"), Some(&Value::px(10.0)));
        assert_eq!(values.get("color"), Some(&Value::ColorValue(crate::css_logic::Color { r: 255, g: 0, b: 0, a: 255 })));
        assert_eq!(values.get("font-size"), Some(&Value::px(12.0)));
    }

    fn get_styled_node_by_tag<'a>(root: &'a StyledNode<'a>, tag_name: &str) -> Option<&'a StyledNode<'a>> {
        if let NodeData::Element { ref name, .. } = root.node.data {
            if name.local.as_ref() == tag_name {
                return Some(root);
            }
        }
        for child in &root.children {
            if let Some(node) = get_styled_node_by_tag(child, tag_name) {
                return Some(node);
            }
        }
        None
    }

    #[test]
    fn test_style_tree() {
        let html = r#"
            <html>
                <body>
                    <div id="test-div">Hello</div>
                    <p>World</p>
                </body>
            </html>
        "#;
        let css = r#"
            div {
                display: block;
            }
            p {
                color: #0000FF;
            }
        "#;
        let dom = parse_html(html.to_string());
        let stylesheet = css_logic::parse(css.to_string());

        let styled_tree = style_tree(&dom.document, &stylesheet);

        let styled_div = get_styled_node_by_tag(&styled_tree, "div").expect("div not found");
        let styled_p = get_styled_node_by_tag(&styled_tree, "p").expect("p not found");

        assert_eq!(styled_div.specified_values.get("display"), Some(&Value::Keyword("block".to_string())));
        assert_eq!(styled_p.specified_values.get("color"), Some(&Value::ColorValue(crate::css_logic::Color { r: 0, g: 0, b: 255, a: 255 })));
    }
}
