use v8::{HandleScope, Local, ObjectTemplate};

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
