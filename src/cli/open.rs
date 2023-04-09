use std::{cell::RefCell, env, rc::Rc};

use structopt::StructOpt;

use crate::{
    core::window::Window,
    javascript::{JsRuntime, JsRuntimeState},
    ui::views::{init_menu, BrowserView},
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

    let mut siv = cursive::default();

    init_menu(&mut siv);

    // let code = r#"
    // async function hello_world() {
    //     print(window.name);
    //     window.name = "test"
    //     print(window.name);
    // }
    // hello_world();
    // "#;

    // let result = runtime.execute_script(code);
    // println!("Result is: {:#?}", result);

    let mut b = BrowserView::named(Rc::new(siv.cb_sink().clone()));
    b.get_mut().navigate_to(start_url);
    siv.add_fullscreen_layer(b);

    siv.run();
}
