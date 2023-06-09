use cursive::View;
use cursive::{view::Resizable, views::Button};
use log::{error, info};

use crate::{
    core::{dom::element::Element, layout::LayoutBox},
    ui::{
        components::TextInputView,
        views::{with_current_browser_view, BrowserView},
    },
};

pub fn render(_: &LayoutBox, element: &Element) -> Box<dyn View> {
    match element
        .attributes
        .get("type")
        .unwrap_or(&"".to_string())
        .as_str()
    {
        "button" | "submit" => {
            let onclick = element
                .attributes
                .get("onclick")
                .unwrap_or(&"".to_string())
                .clone();

            Box::new(Button::new(
                element.attributes.get("value").unwrap_or(&"".to_string()),
                move |s| {
                    let result = with_current_browser_view(s, |b: &mut BrowserView| {
                        b.with_page_view_mut(|p| p.js_runtime.execute_script(onclick.as_str()))
                    });
                    if result.is_none() {
                        error!("failed to run onclick event of button")
                    }
                    match result.unwrap().unwrap() {
                        Ok(message) => {
                            info!("succeeded to run javascript; {}", message);
                        }
                        Err(e) => {
                            error!("failed to run javascript; {}", e);
                        }
                    }
                },
            ))
        }
        _ => Box::new(
            TextInputView::new()
                .content(element.attributes.get("value").unwrap_or(&"".to_string()))
                .min_width(10)
                .max_width(10),
        ),
    }
}
