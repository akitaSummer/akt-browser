use crate::ui::logger::setup_logger;
use cursive::{event::Key, menu, views::Dialog, CursiveRunnable};

pub fn init_menu(siv: &mut CursiveRunnable) {
    setup_logger();
    siv.menubar()
        .add_subtree(
            "Operation",
            menu::Tree::new().leaf("Toggle debug console", |s| {
                s.toggle_debug_console();
            }),
        )
        .add_subtree(
            "Help",
            menu::Tree::new().leaf("About", |s| {
                s.add_layer(Dialog::info(format!(
                    "akt-browser {}",
                    env!("CARGO_PKG_VERSION")
                )))
            }),
        )
        .add_delimiter()
        .add_leaf("Quit", |s| s.quit());

    siv.set_autohide_menu(false);
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
}
