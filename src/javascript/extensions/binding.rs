use v8::{HandleScope, Local, ObjectTemplate, READ_ONLY};

use crate::{core::dom::node::Node, javascript::JsRuntime};

// 创建名为name的obj
pub fn create_object_under<'s>(
    scope: &mut HandleScope<'s>,
    target: Local<v8::Object>,
    name: &'static str,
) -> v8::Local<'s, v8::Object> {
    let template = ObjectTemplate::new(scope);
    let key = v8::String::new(scope, name).unwrap();
    let value = template.new_instance(scope).unwrap();
    target.set(scope, key.into(), value.into());
    value
}

// 设置getter，setter
pub fn set_accessor_to<'s, GetterF, SetterF>(
    scope: &mut HandleScope<'s>,
    target: Local<v8::Object>,
    name: &'static str,
    getter: GetterF,
    setter: SetterF,
) where
    GetterF: Sized
        + Copy
        + Fn(
            &mut v8::HandleScope,
            v8::Local<v8::Name>,
            v8::PropertyCallbackArguments,
            v8::ReturnValue,
        ),
    SetterF: Sized
        + Copy
        + Fn(
            &mut v8::HandleScope,
            v8::Local<v8::Name>,
            v8::Local<v8::Value>,
            v8::PropertyCallbackArguments,
        ),
{
    let key = v8::String::new(scope, name).unwrap();
    target.set_accessor_with_setter(scope, key.into(), getter, setter);
}

// 绑定方法
pub fn set_function_to(
    scope: &mut v8::HandleScope<'_>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let tmpl = v8::FunctionTemplate::new(scope, callback);
    let val = tmpl.get_function(scope).unwrap();
    target.set(scope, key.into(), val.into());
}

// 给obj设置key，value
pub fn set_property_to<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.set(scope, key.into(), value.into());
}

// 设置只读属性
pub fn set_constant_to<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &str,
    cvalue: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.define_own_property(scope, key.into(), cvalue, READ_ONLY);
}
