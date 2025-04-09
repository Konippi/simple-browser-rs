use crate::{
    css_parser::{Unit, Value},
    style::{Display, StyledNode},
};

// To keep the code simple, this code implments only normal flow.
// TODO: Support floats, absolute positioning, and fixed positioning.

#[derive(Debug)]
struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    // Create a new layout box.
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            dimensions: Default::default(),
            box_type,
            children: Vec::new(),
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => {
                panic!("Anonymous block has no style node.")
            }
        }
    }
}

#[derive(Debug)]
enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

#[derive(Clone, Copy, Debug, Default)]
struct Dimensions {
    // Position of the content area
    content: Rectangle,

    // Surrounding edges
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

impl Dimensions {
    // The area covered by the content area plus its padding.
    fn padding_box(self) -> Rectangle {
        self.content.expanded_by(self.padding)
    }
    // The area covered by the content area plus padding and borders.
    fn border_box(self) -> Rectangle {
        self.padding_box().expanded_by(self.border)
    }
    // The area covered by the content area plus padding, borders, and margin.
    fn margin_box(self) -> Rectangle {
        self.border_box().expanded_by(self.margin)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rectangle {
    fn expanded_by(self, edge: EdgeSizes) -> Self {
        Self {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

// Build a layout tree from the style tree.
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
    // Get inline container for the current box.
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

    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => {}  // TODO
            BoxType::AnonymousBlock => {} // TODO
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        // Child width can depend on parent width,
        // so we need to calculate the box's width before laying out its children.
        self.calc_block_width(containing_block);

        // Determine where the box is located within the containing block.
        self.calc_block_position(containing_block);

        // Recursively lay out the children of the box.
        self.layout_block_children();

        // Parent height can depend on child height,
        // so we need to calculate the box's height after laying out its children.
        self.calc_block_height();
    }

    fn calc_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // `width` has initial value `auto`.
        let auto = Value::Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or_else(|| auto.clone());

        let zero = Value::Length(0.0, Unit::Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let border_left = style.lookup("border-left-width", "border", &zero);
        let border_right = style.lookup("border-right-width", "border", &zero);

        let total = sum([
            &margin_left,
            &margin_right,
            &padding_left,
            &padding_right,
            &border_left,
            &border_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_px()));

        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Value::Length(0.0, Unit::Px);
            }
            if margin_right == auto {
                margin_right = Value::Length(0.0, Unit::Px);
            }
        }

        let underflow = containing_block.content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            // If values are defined, adjust margin_right by the underflow.
            (false, false, false) => {
                margin_right =
                    Value::Length(margin_right.to_px() + underflow, Unit::Px);
            }
            // If only margin_right is auto, set underflow to it.
            (false, false, true) => {
                margin_right = Value::Length(underflow, Unit::Px);
            }
            // If only margin_left is auto, set underflow to it.
            (false, true, false) => {
                margin_left = Value::Length(underflow, Unit::Px);
            }
            // If both margins are auto, set them to half of the underflow.
            (false, true, true) => {
                margin_left = Value::Length(underflow / 2.0, Unit::Px);
                margin_right = Value::Length(underflow / 2.0, Unit::Px);
            }
            // If width is auto, any other auto values become 0.
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = Value::Length(underflow, Unit::Px);
                }
                if margin_right == auto {
                    margin_right = Value::Length(underflow, Unit::Px);
                }

                if underflow >= 0.0 {
                    // Expand width to fill the underflow.
                    width = Value::Length(underflow, Unit::Px);
                } else {
                    // Width can't be negative, so adjust the margin_right instead.
                    width = Value::Length(0.0, Unit::Px);
                    margin_right = Value::Length(
                        margin_right.to_px() + underflow,
                        Unit::Px,
                    );
                }
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();
        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();
        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();
    }

    fn calc_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let zero = Value::Length(0.0, Unit::Px);

        let d = &mut self.dimensions;
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom =
            style.lookup("margin-bottom", "margin", &zero).to_px();
        d.border.top = style
            .lookup("border-top-width", "border-width", &zero)
            .to_px();
        d.border.bottom = style
            .lookup("border-bottom-width", "border-width", &zero)
            .to_px();
        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom =
            style.lookup("padding-bottom", "padding", &zero).to_px();
        d.content.x = containing_block.content.x
            + d.margin.left
            + d.border.left
            + d.padding.left;
        d.content.y = containing_block.content.height
            + containing_block.content.y
            + d.margin.top
            + d.border.top
            + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        for child in &mut self.children {
            child.layout(self.dimensions);
            // Increment the height so each child is laid out below the previous one.
            self.dimensions.content.height +=
                child.dimensions.margin_box().height;
        }
    }

    fn calc_block_height(&mut self) {
        if let Some(Value::Length(h, Unit::Px)) =
            self.get_style_node().value("height")
        {
            self.dimensions.content.height = h;
        }
    }
}

fn sum<I>(iter: I) -> f32
where
    I: Iterator<Item = f32>,
{
    iter.fold(0., |acc, x| acc + x)
}
