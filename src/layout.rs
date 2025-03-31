use crate::style::{Display, StyledNode};

#[derive(Debug)]
struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            dimensions: Default::default(),
            box_type,
            children: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

#[derive(Debug, Default)]
struct Dimensions {
    // Position of the content area
    content: Rectangle,

    // Surrounding edges
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

#[derive(Debug, Default)]
struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Default)]
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BoxType::BlockNode(style_node),
        Display::Inline => BoxType::InlineNode(style_node),
        Display::None => panic!("Root node has display: none."),
    });

    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.children.push(build_layout_tree(child)),
            Display::None => {}
        }
    }

    root
}

impl LayoutBox<'_> {
    fn get_inline_container(&mut self) -> &mut Self {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) => {
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::AnonymousBlock,
                        ..
                    }) => {}
                    _ => self
                        .children
                        .push(LayoutBox::new(BoxType::AnonymousBlock)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}
