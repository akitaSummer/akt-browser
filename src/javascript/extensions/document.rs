use crate::{
    core::dom::node::{Node, NodeType},
    javascript::JsRuntimeState,
};

use super::binding::{set_accessor_to, set_constant_to, set_function_to, set_property_to};

use log::{error, info, trace};
use std::ffi::c_void;

// node
fn create_v8_node<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Object> {
    let template = v8::ObjectTemplate::new(scope);

    template.set_internal_field_count(1);

    template.new_instance(scope).unwrap()
}

type NodeRefTarget<'a> = &'a mut Box<Node>;

fn to_v8_element<'s>(
    scope: &mut v8::HandleScope<'s>,
    tag_name: &str,
    attributes: Vec<(String, String)>,
    node_rust: NodeRefTarget,
) -> v8::Local<'s, v8::Object> {
    let node = to_v8_node(scope, node_rust);

    {
        let tag_name = v8::String::new(scope, tag_name).unwrap();
        set_constant_to(scope, node, "tagName", tag_name.into());
    }
    {
        for (key, value) in attributes {
            let value = v8::String::new(scope, value.as_str()).unwrap();
            set_constant_to(scope, node, key.as_str(), value.into());
        }
    }
    {
        set_accessor_to(
            scope,
            node,
            "innerHTML",
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  args: v8::PropertyCallbackArguments,
                  mut rv: v8::ReturnValue| {
                let this = args.this();
                let node = to_linked_rust_node(scope, this);

                let ret = v8::String::new(scope, node.inner_html().as_str()).unwrap();
                rv.set(ret.into());
            },
            move |scope: &mut v8::HandleScope,
                  _key: v8::Local<v8::Name>,
                  value: v8::Local<v8::Value>,
                  args: v8::PropertyCallbackArguments| {
                let this = args.this();
                let node = to_linked_rust_node(scope, this);
                if let Err(e) = node.set_inner_html(value.to_rust_string_lossy(scope)) {
                    error!("failed to set innerHTML; {}", e);
                }
                request_rerender(scope, "setter of innerHTML");
            },
        );
    }

    node
}

pub fn create_document_object<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Object> {
    let document = create_v8_node(scope);

    {
        set_accessor_to(
            scope,
            document,
            "all",
            |scope: &mut v8::HandleScope,
             _key: v8::Local<v8::Name>,
             _args: v8::PropertyCallbackArguments,
             mut rv: v8::ReturnValue| {
                let document = match JsRuntimeState::document(scope) {
                    Some(_document) => _document,
                    None => {
                        error!("failed to get document reference; document is None");
                        return;
                    }
                };
                let mut document = document.borrow_mut();

                // 获取所有node
                let document_element = &mut document.document_element;

                // 为node绑定innerHTML的方法
                let mut f = |n: &mut Box<Node>| -> Option<v8::Local<v8::Value>> {
                    let (tag_name, attributes) = match n.node_type {
                        NodeType::Element(ref e) => (e.tag_name.clone(), e.attributes()),
                        _ => return None,
                    };
                    Some(to_v8_element(scope, tag_name.as_str(), attributes, n).into())
                };

                // 为所有node绑定innerHTML方法
                let all: Vec<v8::Local<v8::Value>> = map_mut(document_element, &mut f)
                    .into_iter()
                    .filter_map(|n| n)
                    .collect();

                // 所有node的数组
                let all = v8::Array::new_with_elements(scope, all.as_slice());

                // 返回给v8
                rv.set(all.into());
            },
            |_scope: &mut v8::HandleScope,
             _key: v8::Local<v8::Name>,
             _value: v8::Local<v8::Value>,
             _args: v8::PropertyCallbackArguments| {},
        );
    }
    {
        // getElementById
        set_function_to(
            scope,
            document,
            "getElementById",
            |scope: &mut v8::HandleScope,
             args: v8::FunctionCallbackArguments,
             mut retval: v8::ReturnValue| {
                let id = args
                    .get(0)
                    .to_string(scope)
                    .unwrap()
                    .to_rust_string_lossy(scope);
                let document = match JsRuntimeState::document(scope) {
                    Some(_document) => _document,
                    None => {
                        error!("failed to get document reference; document is None");
                        return;
                    }
                };
                let mut document = document.borrow_mut();

                // all nodes
                let document_element = &mut document.document_element;

                // 找node
                let mut f = |n: &mut Box<Node>| -> Option<v8::Local<v8::Value>> {
                    let (tag_name, attributes) = match n.node_type {
                        NodeType::Element(ref e) => {
                            if e.id().map(|eid| eid.to_string() == id).unwrap_or(false) {
                                (e.tag_name.clone(), e.attributes())
                            } else {
                                return None;
                            }
                        }
                        _ => return None,
                    };
                    Some(to_v8_element(scope, tag_name.as_str(), attributes, n).into())
                };

                // 找node
                let element: v8::Local<v8::Value> = map_mut(document_element, &mut f)
                    .into_iter()
                    .find_map(|n| n)
                    .unwrap_or(v8::undefined(scope).into());

                // 返回
                retval.set(element.into());
            },
        );
    }

    document
}

// 创建dom树
pub fn initialize_dom<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) {
    let document = create_document_object(scope);
    set_property_to(scope, global, "document", document.into());
}

// rust中的node引用映射到v8对象
fn set_node_internal_ref<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_rust: NodeRefTarget,
    node_v8: v8::Local<v8::Object>,
) {
    let boxed_ref = Box::new(node_rust);
    // 获取裸指针地址
    let addr = Box::leak(boxed_ref) as *mut NodeRefTarget as *mut c_void;
    // 将指针映射到v8中
    let v8_ext = v8::External::new(scope, addr);
    // 获取到其v8的的格式
    let target_node_ref_v8: v8::Local<v8::Value> = v8_ext.into();
    // 将node_v8值变为rust_node
    node_v8.set_internal_field(0, target_node_ref_v8);
}

// 将v8node转化为rust_node
fn to_linked_rust_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_v8: v8::Local<v8::Object>,
) -> &'s mut NodeRefTarget<'s> {
    // 获取整个node的value
    let node_v8 = node_v8.get_internal_field(scope, 0).unwrap();
    // 获取其v8中的指针映射
    let node = unsafe { v8::Local::<v8::External>::cast(node_v8) };
    // 获取rust的指针形式
    let node = node.value() as *mut NodeRefTarget;
    // 返回可修改的引用
    unsafe { &mut *node }
}

// 将一个rust_node改变为v8的obj
fn to_v8_node<'s>(
    scope: &mut v8::HandleScope<'s>,
    node_rust: NodeRefTarget,
) -> v8::Local<'s, v8::Object> {
    let node_v8 = create_v8_node(scope);

    set_node_internal_ref(scope, node_rust, node_v8);

    node_v8
}

// 将node及child执行fn
fn map_mut<T, F>(node: NodeRefTarget, f: &mut F) -> Vec<T>
where
    F: FnMut(&mut Box<Node>) -> T,
{
    let mut v: Vec<T> = vec![];

    for child in &mut node.children {
        v.push(f(child));
        v.extend(map_mut(child, f));
    }

    v.push(f(node));
    v
}

// 重新渲染
fn request_rerender<'s>(scope: &mut v8::HandleScope<'s>, caller: &'static str) {
    let pv_api_handler = match JsRuntimeState::pv_api_handler(scope) {
        Some(_p) => _p,
        None => {
            error!("failed to get document reference; pv_api_handler is None");
            return;
        }
    };
    match pv_api_handler.request_rerender() {
        Ok(_) => {
            info!("re-render requested from {}", caller);
        }
        Err(e) => {
            error!("failed to request alert(); {}", e);
        }
    };
}
