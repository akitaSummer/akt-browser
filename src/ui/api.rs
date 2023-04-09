use std::{error::Error, rc::Rc};

use cursive::CbSink;
use log::{error, info};

use super::{components::alert, views::with_current_page_view};

pub struct PageViewAPIHandler {
    ui_cb_sink: Rc<CbSink>,
}

impl PageViewAPIHandler {
    pub fn new(ui_cb_sink: Rc<CbSink>) -> Self {
        Self {
            ui_cb_sink: ui_cb_sink,
        }
    }

    pub fn alert(&self, message: String) -> Result<(), Box<dyn Error>> {
        self.ui_cb_sink
            .send(Box::new(move |s: &mut cursive::Cursive| {
                alert(s, "from JavaScript".to_string(), message);
            }))?;
        Ok(())
    }

    pub fn request_rerender(&self) -> Result<(), Box<dyn Error>> {
        self.ui_cb_sink
            .send(Box::new(move |s: &mut cursive::Cursive| {
                with_current_page_view(s, |v| {
                    info!("re-rendering started");
                    match v.render_document() {
                        Ok(_) => info!("re-rendering finished"),
                        Err(e) => error!("re-rendering failed; {}", e),
                    }
                });
            }))?;
        Ok(())
    }
}
