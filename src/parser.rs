use std::collections::HashMap;

use crate::dom::{AttributeMap, Node};

pub struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    // Parse an HTML document, returning the root element.
    pub fn parse(source: String) -> Node {
        let mut nodes = Self {
            pos: 0,
            input: source,
        }
        .parse_nodes();

        // If the document contains a root element, return it.
        // Otherwise, create one.
        if nodes.len() == 1 {
            return nodes.remove(0);
        }
        Node::new_by_element("html".to_string(), HashMap::new(), nodes)
    }

    // Parse nodes.
    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    // Parse a single node.
    fn parse_node(&mut self) -> Node {
        if self.starts_with("<") {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }

    // Parse a single element.
    fn parse_element(&mut self) -> Node {
        // Opening tag.
        self.expect("<");
        let tag_name = self.parse_name();
        let attributes = self.parse_attributes();
        self.expect(">");

        // Children.
        let children = self.parse_nodes();

        // Closing tag.
        self.expect("</");
        self.expect(tag_name.as_str());
        self.expect(">");

        Node::new_by_element(tag_name, attributes, children)
    }

    // Parse a text node.
    fn parse_text(&mut self) -> Node {
        Node::new_by_text(self.consume_chars_while(|c| c != '<'))
    }

    // Parse attributes.
    fn parse_attributes(&mut self) -> AttributeMap {
        let mut attributes = AttributeMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attribute();
            attributes.insert(name, value);
        }
        attributes
    }

    // Parse a single attribute.
    fn parse_attribute(&mut self) -> (String, String) {
        let name = self.parse_name();
        self.expect("=");
        let value = self.parse_attribute_value();
        (name, value)
    }

    // Parse an attribute value.
    fn parse_attribute_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_chars_while(|c| c != open_quote);
        let close_quote = self.consume_char();
        assert_eq!(open_quote, close_quote);
        value
    }

    // Parse a tag or attribute name.
    fn parse_name(&mut self) -> String {
        self.consume_chars_while(
            |c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'),
        )
    }

    // Consume whitespace.
    fn consume_whitespace(&mut self) {
        self.consume_chars_while(char::is_whitespace);
    }

    // Consume characters while the condition is true.
    fn consume_chars_while(
        &mut self,
        condition: impl Fn(char) -> bool,
    ) -> String {
        let mut result = String::new();
        while !self.eof() && condition(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    // Consume a character.
    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }

    // Read the next character from the input.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Check if the input starts with a given string.
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // If the exact string is found, move the position forward.
    // Otherwise, panic.
    fn expect(&mut self, s: &str) {
        if self.starts_with(s) {
            self.pos += s.len();
        } else {
            panic!(
                "Expected {:?} at byte {} but it was not found",
                s, self.pos
            );
        }
    }

    // Check if the input is at the end
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
