mod binding;

use std::borrow::Borrow;

use lazy_static::lazy_static;
use log::{error, trace};
use v8::{
    Context, ContextScope, ExternalReference, ExternalReferences, FunctionCallbackArguments,
    Global, HandleScope, Local, MapFnTo, ReturnValue,
};

use self::binding::*;
use super::{execute_script, JsRuntimeState};

const GLUE: &str = include_str!("glue.js");

lazy_static! {
    pub static ref EXTERNAL_REFERNCES: ExternalReferences =
        ExternalReferences::new(&[ExternalReference {
            function: MapFnTo::map_fn_to(print),
        }]);
}

pub struct Extensions;

impl Extensions {
    pub fn install(scope: &mut HandleScope, context: Local<Context>) {
        // binding window
        let global = context.global(scope);
        let scope = &mut v8::ContextScope::new(scope, context);
        initialize_window(scope, global);

        // binding print
        {
            let bindings = v8::Object::new(scope);

            let name = v8::String::new(scope, "print").unwrap();
            let func = v8::Function::new(scope, print).unwrap();
            bindings.set(scope, name.into(), func.into()).unwrap();

            if let Ok(result) = execute_script(scope, GLUE) {
                let func = v8::Local::<v8::Function>::try_from(result).unwrap();
                let v = v8::undefined(scope).into();
                let args = [bindings.into()];
                func.call(scope, v, &args).unwrap();
            };
        }
    }
}

fn print(scope: &mut HandleScope, args: FunctionCallbackArguments, mut rv: ReturnValue) {
    let result: serde_json::Value = serde_v8::from_v8(scope, args.get(0)).unwrap();
    println!("Rust say: {:#?}", result);
    rv.set(serde_v8::to_v8(scope, result).unwrap());
}

pub fn initialize_window<'s>(
    scope: &mut ContextScope<'s, HandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let window = create_object_under(scope, global, "window");

    // `name` property
    set_accessor_to(
        scope,
        window,
        "name",
        |scope: &mut v8::HandleScope,
         key: v8::Local<v8::Name>,
         _args: v8::PropertyCallbackArguments,
         mut rv: v8::ReturnValue| {
            trace!("Read access to: {}", key.to_rust_string_lossy(scope));

            let window = JsRuntimeState::window(scope);
            let window = window.unwrap();
            let window = window.borrow_mut();

            let value = window.name.as_str();

            // println!("name is: {:#?}", value);

            rv.set(v8::String::new(scope, value).unwrap().into());
        },
        |scope: &mut v8::HandleScope,
         key: v8::Local<v8::Name>,
         value: v8::Local<v8::Value>,
         _args: v8::PropertyCallbackArguments| {
            trace!("Write access to: {}", key.to_rust_string_lossy(scope));

            let window = JsRuntimeState::window(scope);
            let window = window.unwrap();
            let mut window = window.borrow_mut();

            let value = value.to_rust_string_lossy(scope);

            // println!("new name is: {:#?}", value);

            window.name = value;
        },
    );

    window
}
