use std::{cell::RefCell, env, rc::Rc};

use structopt::StructOpt;

use crate::{
    core::window::Window,
    javascript::{JsRuntime, JsRuntimeState},
    utils,
};

#[derive(StructOpt, Debug)]
pub struct Opts {
    pub url: Option<String>,
}

pub fn run(opts: Opts) {
    let start_url = opts
        .url
        .and_then(|u| Some(utils::resolves_path(env::current_dir().unwrap(), u)))
        .unwrap_or("https://akitasummer.github.io".to_string());

    print!("url is : {}\n", start_url);

    let mut runtime = JsRuntime::new(None);

    let window = Rc::new(RefCell::new(Window {
        name: "akitasummer".to_string(),
    }));

    JsRuntimeState::set_window(&mut runtime.isolate, window);

    let code = r#"
    async function hello_world() {
        print(window.name);
        window.name = "test"
        print(window.name);
    }
    hello_world();
    "#;

    let result = runtime.execute_script(code);
    println!("Result is: {:#?}", result);
}
