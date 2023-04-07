use super::node::{Node, NodeType};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub url: String,
    pub document_uri: String,
    pub document_element: Box<Node>,
}

impl Document {
    pub fn new(url: String, document_uri: String, document_element: Box<Node>) -> Document {
        Document {
            url: url,
            document_uri: document_uri,
            document_element: document_element,
        }
    }

    pub fn collect_tag_inners(&self, tag_name: &str) -> Vec<String> {
        Self::intl(&self.document_element, tag_name)
    }

    fn intl(node: &Box<Node>, tag_name: &str) -> Vec<String> {
        if let NodeType::Element(ref element) = node.node_type {
            if element.tag_name.as_str() == tag_name {
                return vec![node.inner_text()];
            }
        }

        node.children
            .iter()
            .clone()
            .into_iter()
            .map(|child| Self::intl(child, tag_name))
            .collect::<Vec<Vec<String>>>()
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn get_script_inners(&self) -> Vec<String> {
        self.collect_tag_inners("script")
    }

    pub fn get_style_inners(&self) -> Vec<String> {
        self.collect_tag_inners("style")
    }
}
