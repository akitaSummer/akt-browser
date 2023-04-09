use cursive::{traits::Finder, view::ViewWrapper, views::LinearLayout, CbSink, Cursive, With};
use std::{cell::RefCell, rc::Rc};

use crate::{
    core::{
        dom::document::Document,
        layout::{to_layout_document, LayoutDocument},
        style::{to_styled_document, StyledDocument},
        window::Window,
    },
    javascript::{JsRuntime, JsRuntimeState},
    ui::{
        api::PageViewAPIHandler,
        render::{to_element_container, ElementContainer},
    },
};
use log::{error, info};
use thiserror::Error;

use super::PAGE_VIEW_NAME;

#[derive(Error, Debug, PartialEq)]
pub enum PageError {
    #[error("failed to render; no document exists")]
    NoDocumentError,

    #[error("failed to render; javascript execution failed: {0:?}")]
    JavaScriptError(serde_json::Value),
}

pub struct PageView {
    window: Option<Rc<RefCell<Window>>>,
    document: Option<Rc<RefCell<Document>>>,

    view: ElementContainer,

    pub js_runtime: JsRuntime,
}

impl PageView {
    pub fn new(ui_cb_sink: Rc<CbSink>) -> Self {
        (Self {
            window: None,
            document: None,

            view: ElementContainer::vertical(),

            js_runtime: JsRuntime::new(None),
        })
        .with(|v| {
            JsRuntimeState::set_pv_api_handler(
                &mut v.js_runtime.isolate,
                Rc::new(PageViewAPIHandler::new(ui_cb_sink)),
            );
        })
    }

    pub fn init_page(&mut self, document: Document) -> Result<(), PageError> {
        let window = Rc::new(RefCell::new(Window {
            name: "".to_string(),
        }));

        let document = Rc::new(RefCell::new(document));

        self.window = Some(window.clone());
        self.document = Some(document.clone());

        let isolate = &mut self.js_runtime.isolate;
        JsRuntimeState::set_window(isolate, window.clone());
        JsRuntimeState::set_document(isolate, document.clone());

        self.render_document()?;

        self.execute_inline_scripts()?;

        Ok(())
    }

    pub fn render_document(&mut self) -> Result<(), PageError> {
        let document = match &self.document {
            Some(w) => w,
            None => return Err(PageError::NoDocumentError),
        };
        let document = &*document.borrow_mut();
        let styled: StyledDocument = to_styled_document(document);
        let layout: LayoutDocument = to_layout_document(styled);

        self.view = to_element_container(&layout.top_box);

        Ok(())
    }

    fn execute_inline_scripts(&mut self) -> Result<(), PageError> {
        let scripts = {
            let document = match &self.document {
                Some(w) => w,
                None => return Err(PageError::NoDocumentError),
            };
            let document = document.borrow_mut();
            document.get_script_inners()
        };

        for script in scripts {
            match self.js_runtime.execute_script(script.as_str()) {
                Ok(s) => {
                    info!("javascript execution succeeded; {}", s);
                }
                Err(e) => return Err(PageError::JavaScriptError(e)),
            };
        }
        Ok(())
    }
}

impl ViewWrapper for PageView {
    type V = LinearLayout;

    fn with_view<F, R>(&self, f: F) -> ::std::option::Option<R>
    where
        F: FnOnce(&Self::V) -> R,
    {
        Some(f(&self.view))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
    where
        F: ::std::ops::FnOnce(&mut Self::V) -> R,
    {
        Some(f(&mut self.view))
    }

    fn into_inner(self) -> ::std::result::Result<Self::V, Self>
    where
        Self::V: ::std::marker::Sized,
    {
        Ok(self.view)
    }
}

pub fn with_current_page_view<Output, F>(s: &mut Cursive, f: F) -> Option<Output>
where
    F: FnOnce(&mut PageView) -> Output,
{
    s.screen_mut().call_on_name(PAGE_VIEW_NAME, f)
}
