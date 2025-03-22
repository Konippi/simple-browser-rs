use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

impl Node {
    // Create a node by given text.
    pub fn new_by_text(text: String) -> Self {
        Self {
            children: vec![],
            node_type: NodeType::Text(text),
        }
    }

    // Create a node by given element data.
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
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttributeMap,
}

pub type AttributeMap = HashMap<String, String>;

impl ElementData {
    // Get id.
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    // Get classes.
    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classes) => classes.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}
