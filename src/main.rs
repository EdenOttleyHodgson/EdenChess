use std::{sync::mpsc::channel, thread};

use derive_more::Display;
use env_logger::Env;
use ui::init_ui;

mod model;
mod ui;

use log::*;

use crate::model::init_model;
#[derive(Debug, Display)]
pub enum ControlMsg {
    Debug,
}
fn main() {
    let log_env = Env::default().filter_or("debug", "debug");
    env_logger::init_from_env(log_env);
    debug!("Debug lgo");
    let (ui_send, model_recv) = channel();
    let (model_send, ui_recv) = channel();

    let model_thread_handle = thread::spawn(move || {
        init_model(ui_send, ui_recv);
    });
    init_ui(model_send, model_recv);
}
