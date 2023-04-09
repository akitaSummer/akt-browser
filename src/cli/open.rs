use std::{env, rc::Rc};

use structopt::StructOpt;

use crate::{
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

    let mut siv = cursive::default();

    init_menu(&mut siv);

    let mut b = BrowserView::named(Rc::new(siv.cb_sink().clone()));
    b.get_mut().navigate_to(start_url);
    siv.add_fullscreen_layer(b);

    siv.run();
}
