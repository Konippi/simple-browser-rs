use std::collections::HashMap;

use crate::{
    css_parser::{
        Rule, Selector, SimpleSelector, Specificity, StyleSheet, Value,
    },
    dom::{ElementData, Node, NodeType},
};

type PropertyMap = HashMap<String, Value>;

#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

#[derive(PartialEq)]
pub enum Display {
    Inline,
    Block,
    None,
}

impl<'a> StyledNode<'a> {
    // Get the value by given property name.
    #[inline]
    pub fn value(&self, property_name: &str) -> Option<Value> {
        self.specified_values.get(property_name).cloned()
    }

    // Get the value by given property name or fallback property name.
    // If the value is not found, return the default value.
    pub fn lookup(
        &self,
        property_name: &str,
        fallback_name: &str,
        default: &Value,
    ) -> Value {
        self.value(property_name).unwrap_or_else(|| {
            self.value(fallback_name).unwrap_or_else(|| default.clone())
        })
    }

    // Get the display value.
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}

// Apply a stylesheet to an entire DOM tree.
fn style_tree<'a>(
    root: &'a Node,
    stylesheet: &'a StyleSheet,
) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(elem, stylesheet),
            NodeType::Text(_) => HashMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}

// Apply styles to a single element.
fn specified_values(
    elem: &ElementData,
    stylesheet: &StyleSheet,
) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    rules.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    return values;
}

type MatchedRule<'a> = (Specificity, &'a Rule);

// Find all CSS rules that match the given element.
fn matching_rules<'a>(
    elem: &ElementData,
    stylesheet: &'a StyleSheet,
) -> Vec<MatchedRule<'a>> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}

// Match a CSS rule to an element.
fn match_rule<'a>(
    elem: &ElementData,
    rule: &'a Rule,
) -> Option<MatchedRule<'a>> {
    rule.selectors
        .iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

// Check if a selector matches an element.
#[inline]
fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(simple) => matches_simple_selector(elem, simple),
    }
}

// Check if a simple selector matches an element.
fn matches_simple_selector(
    elem: &ElementData,
    selector: &SimpleSelector,
) -> bool {
    // Check type selector.
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check id selector.
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selector.
    if !selector
        .class
        .iter()
        .any(|class| elem.classes().contains(class.as_str()))
    {
        return false;
    }

    true
}
