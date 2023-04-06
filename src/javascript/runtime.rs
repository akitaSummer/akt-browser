use std::borrow::Borrow;

use once_cell::sync::OnceCell;

use v8::{CreateParams, HandleScope, Isolate, OwnedIsolate, Script, ScriptOrigin, TryCatch, V8};

use super::{extensions::Extensions, state::JsRuntimeState, EXTERNAL_REFERNCES};

pub type LocalValue<'a> = v8::Local<'a, v8::Value>;

pub struct JsRuntime {
    pub isolate: OwnedIsolate,
}

impl JsRuntime {
    pub fn new(snapshot: Option<Vec<u8>>) -> Self {
        JsRuntime::init();

        let mut params = CreateParams::default().external_references(&**EXTERNAL_REFERNCES);
        let mut initialized = false;
        if let Some(snapshot) = snapshot {
            params = params.snapshot_blob(snapshot);
            initialized = true;
        }
        let isolate = Isolate::new(params);
        JsRuntime::init_isolate(isolate, initialized)
    }

    fn init() {
        // v8只能实例化一次
        static V8_INSTANCE: OnceCell<()> = OnceCell::new();
        V8_INSTANCE.get_or_init(|| {
            let platform = v8::new_default_platform(0, false).make_shared();
            V8::initialize_platform(platform);
            V8::initialize();
        });
    }

    fn init_isolate(mut isolate: OwnedIsolate, initialized: bool) -> Self {
        let state = JsRuntimeState::new(&mut isolate);
        isolate.set_slot(state);
        if !initialized {
            let context = JsRuntimeState::get_context(&mut isolate);
            let scope = &mut HandleScope::with_context(&mut isolate, context);
            Extensions::install(scope, scope.get_current_context());
        };
        JsRuntime { isolate }
    }

    pub fn execute_script(
        &mut self,
        code: impl AsRef<str>,
    ) -> Result<serde_json::Value, serde_json::Value> {
        let context = JsRuntimeState::get_context(&mut self.isolate);
        let handle_scope = &mut HandleScope::with_context(&mut self.isolate, context);
        match execute_script(handle_scope, code) {
            Ok(value) => Ok(serde_v8::from_v8(handle_scope, value).unwrap()),
            Err(error) => Err(serde_v8::from_v8(handle_scope, error).unwrap()),
        }
    }
}

pub fn execute_script<'a>(
    scope: &mut HandleScope<'a>,
    code: impl AsRef<str>,
) -> Result<LocalValue<'a>, LocalValue<'a>> {
    let scope = &mut TryCatch::new(scope);
    let source = v8::String::new(scope, code.as_ref()).unwrap();
    let origin = create_origin(scope, "dummy.js");

    Script::compile(scope, source, Some(&origin))
        .and_then(|script| script.run(scope))
        .map_or_else(|| Err(scope.stack_trace().unwrap()), Ok)
}

fn create_origin<'a>(scope: &mut HandleScope<'a>, filename: impl AsRef<str>) -> ScriptOrigin<'a> {
    let name: LocalValue = v8::String::new(scope, filename.as_ref()).unwrap().into();
    ScriptOrigin::new(
        scope,
        name.clone(),
        0,
        0,
        false,
        0,
        name,
        false,
        false,
        false,
    )
}
