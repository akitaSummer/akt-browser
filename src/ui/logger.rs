use cursive::logger;
use log::{set_max_level, Level};

// 设置debug
pub fn setup_logger() {
    logger::init();
    set_max_level(Level::Info.to_level_filter());
}
