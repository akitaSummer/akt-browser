use std::{cell::RefCell, rc::Rc};

use v8::{HandleScope, Isolate};

use crate::{
    core::{dom::document::Document, window::Window},
    ui::api::PageViewAPIHandler,
};

type GlobalContext = v8::Global<v8::Context>;

type JsRuntimeStateRef = Rc<RefCell<JsRuntimeState>>;

pub struct JsRuntimeState {
    pub context: Option<v8::Global<v8::Context>>,
    pub window: Option<Rc<RefCell<Window>>>,
    pub document: Option<Rc<RefCell<Document>>>,
    pub pv_api_handler: Option<Rc<PageViewAPIHandler>>,
}

impl JsRuntimeState {
    pub fn new(isolate: &mut Isolate) -> JsRuntimeStateRef {
        let context = {
            let handle_scope = &mut HandleScope::new(isolate);
            let context = v8::Context::new(handle_scope);
            v8::Global::new(handle_scope, context)
        };

        Rc::new(RefCell::new(JsRuntimeState {
            context: Some(context),
            window: None,
            document: None,
            pv_api_handler: None,
        }))
    }

    pub fn get_context(isolate: &mut Isolate) -> GlobalContext {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        let ctx = &state.borrow().context;
        ctx.as_ref().unwrap().clone()
    }

    pub fn drop_context(isolate: &mut Isolate) {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        state.borrow_mut().context.take();
    }

    pub fn get_handle_scope(isolate: &mut Isolate) -> HandleScope {
        let context = Self::get_context(isolate);
        HandleScope::with_context(isolate, context)
    }
}

impl JsRuntimeState {
    pub fn window(isolate: &v8::Isolate) -> Option<Rc<RefCell<Window>>> {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        let state = state.borrow();
        state.window.clone()
    }

    pub fn get_window(isolate: &mut Isolate) -> Option<Rc<RefCell<Window>>> {
        Self::window(isolate)
    }
    pub fn set_window(isolate: &mut Isolate, window: Rc<RefCell<Window>>) {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        state.borrow_mut().window = Some(window);
    }
}

impl JsRuntimeState {
    pub fn document(isolate: &v8::Isolate) -> Option<Rc<RefCell<Document>>> {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        let state = state.borrow();
        state.document.clone()
    }

    pub fn get_document(isolate: &mut Isolate) -> Option<Rc<RefCell<Document>>> {
        Self::document(isolate)
    }
    pub fn set_document(isolate: &mut Isolate, document: Rc<RefCell<Document>>) {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        state.borrow_mut().document = Some(document);
    }
}

impl JsRuntimeState {
    pub fn pv_api_handler(isolate: &v8::Isolate) -> Option<Rc<PageViewAPIHandler>> {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        let state = state.borrow();
        state.pv_api_handler.clone()
    }

    pub fn get_pv_api_handler(isolate: &mut Isolate) -> Option<Rc<PageViewAPIHandler>> {
        Self::pv_api_handler(isolate)
    }
    pub fn set_pv_api_handler(isolate: &mut Isolate, view_api_handler: Rc<PageViewAPIHandler>) {
        let state = isolate.get_slot::<JsRuntimeStateRef>().unwrap().clone();
        state.borrow_mut().pv_api_handler = Some(view_api_handler);
    }
}
