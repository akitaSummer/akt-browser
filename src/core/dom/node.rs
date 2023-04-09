use std::error::Error;

use crate::core::html::parse_without_normalziation;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Element(super::element::Element),
    Text(super::text::Text),
}

#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Box<Node>>,
}

impl Node {
    pub fn inner_text(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| match &node.node_type {
                NodeType::Text(t) => t.data.clone(),
                _ => node.inner_text(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn inner_html(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn set_inner_html(&mut self, html: String) -> Result<(), Box<dyn Error>> {
        let node = parse_without_normalziation(html.as_bytes().into())?;
        self.children = node;
        Ok(())
    }
}

impl ToString for Node {
    fn to_string(&self) -> String {
        match self.node_type {
            NodeType::Element(ref e) => {
                let attrs = e
                    .attributes
                    .iter()
                    .clone()
                    .into_iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(" ");
                let children = self
                    .children
                    .iter()
                    .clone()
                    .into_iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>()
                    .join("");
                if attrs != "" {
                    format!("<{} {}>{}</{}>", e.tag_name, attrs, children, e.tag_name)
                } else {
                    format!("<{}>{}</{}>", e.tag_name, children, e.tag_name)
                }
            }
            NodeType::Text(ref t) => t.data.clone(),
        }
    }
}
