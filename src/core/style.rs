use super::{
    css::{self, CSSValue, Stylesheet},
    dom::{
        document::Document,
        node::{Node, NodeType},
    },
};
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, CSSValue>;

#[derive(Debug, PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

#[derive(Debug)]
pub struct StyledDocument<'a> {
    pub document_element: StyledNode<'a>,
}

#[derive(Debug, PartialEq)]
pub struct StyledNode<'a> {
    pub node_type: &'a NodeType,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

impl<'a> StyledNode<'a> {
    pub fn display(&self) -> Display {
        match self.properties.get("display") {
            Some(CSSValue::Keyword(s)) => match s.as_str() {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}

const DEFAULT_STYLESHEET: &str = r#"
script, style {
    display: none;
}
p, div {
    display: block;
}
"#;

pub fn to_styled_document<'a>(document: &'a Document) -> StyledDocument<'a> {
    let styles = format!(
        "{}\n{}",
        DEFAULT_STYLESHEET.to_string(),
        document.get_style_inners().join("\n")
    );
    let stylesheet = css::parse(styles).unwrap_or(Stylesheet::new(vec![]));
    let document_element = to_styled_node(&document.document_element, &stylesheet);

    StyledDocument {
        document_element: document_element,
    }
}

fn to_styled_node<'a>(node: &'a Box<Node>, stylesheet: &Stylesheet) -> StyledNode<'a> {
    let mut props = PropertyMap::new();
    let children = to_styled_nodes(&node.children, stylesheet);

    for matched_rule in stylesheet.rules.iter().filter(|r| r.matches(node)) {
        for declaration in &matched_rule.declarations {
            props.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    StyledNode {
        node_type: &node.node_type,
        properties: props,
        children: children,
    }
}

fn to_styled_nodes<'a>(nodes: &'a Vec<Box<Node>>, stylesheet: &Stylesheet) -> Vec<StyledNode<'a>> {
    nodes
        .iter()
        .map(|x| to_styled_node(x, stylesheet))
        .collect()
}
