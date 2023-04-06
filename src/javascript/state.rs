use std::{cell::RefCell, rc::Rc};

use v8::{HandleScope, Isolate};

use crate::core::window::Window;

type GlobalContext = v8::Global<v8::Context>;

type JsRuntimeStateRef = Rc<RefCell<JsRuntimeState>>;

pub struct JsRuntimeState {
    pub context: Option<v8::Global<v8::Context>>,
    pub window: Option<Rc<RefCell<Window>>>,
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
