mod a;
mod i;
mod input;

use crate::core::{
    dom::element::Element,
    dom::node::NodeType,
    layout::{BoxProps, BoxType, LayoutBox},
};
use cursive::{
    views::{LinearLayout, TextView},
    View,
};

pub type ElementContainer = LinearLayout;

pub fn to_element_container<'a>(layout: &LayoutBox<'a>) -> ElementContainer {
    let mut container = match layout.box_type {
        BoxType::NoneBox => {
            return LinearLayout::horizontal();
        }
        BoxType::BlockBox => LinearLayout::vertical(),
        BoxType::InlineBox | BoxType::AnonymousBox => LinearLayout::horizontal(),
    };

    let elements = match layout.box_props {
        Some(BoxProps {
            node_type: NodeType::Element(ref element),
            ..
        }) => match element.tag_name.as_str() {
            "a" => vec![a::render(layout, element)],
            "i" => vec![i::render(layout, element)],
            "input" => vec![input::render(layout, element)],
            _ => layout
                .children
                .iter()
                .map(|child| Box::new(to_element_container(child)) as Box<dyn View>)
                .collect(),
        },
        Some(BoxProps {
            node_type: NodeType::Text(ref t),
            ..
        }) => {
            let text_to_display = t.data.clone();
            let text_to_display = text_to_display.replace("\n", "");
            let text_to_display = text_to_display.trim();
            if text_to_display != "" {
                vec![Box::new(TextView::new(text_to_display)) as Box<dyn View>]
            } else {
                vec![]
            }
        }
        _ => layout
            .children
            .iter()
            .map(|child| Box::new(to_element_container(child)) as Box<dyn View>)
            .collect(),
    };
    for child in elements {
        container.add_child(child);
    }

    container
}
