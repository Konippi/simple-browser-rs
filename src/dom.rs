use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Node {
    children: Vec<Node>,
    node_type: NodeType,
}

impl Node {
    pub fn new_by_text(text: String) -> Self {
        Self {
            children: vec![],
            node_type: NodeType::Text(text),
        }
    }

    pub fn new_by_element(
        tag_name: String,
        attributes: AttributeMap,
        children: Vec<Node>,
    ) -> Self {
        Self {
            children,
            node_type: NodeType::Element(ElementData {
                tag_name,
                attributes,
            }),
        }
    }
}

#[derive(Debug)]
enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug)]
struct ElementData {
    tag_name: String,
    attributes: AttributeMap,
}

pub type AttributeMap = HashMap<String, String>;

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classes) => classes.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}
