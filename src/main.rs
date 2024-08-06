use std::{io, sync::mpsc::channel, thread};

use derive_more::Display;
use flexi_logger::{FileSpec, Logger, WriteMode};
use ui::init_ui;

mod control;
mod model;
mod ui;

use log::*;

use crate::model::init_model;

fn main() -> io::Result<()> {
    std::process::Command::new("rm")
        .args(["*.log"])
        .output()
        .expect("Failed to execute process");
    let _logger = if cfg!(test) {
        Logger::try_with_str("debug")
            .unwrap()
            .log_to_stdout()
            .start()
            .unwrap();
    } else {
        Logger::try_with_str("debug")
            .unwrap()
            .log_to_file(FileSpec::default())
            .write_mode(WriteMode::BufferAndFlush)
            .start()
            .unwrap();
    };
    debug!("Debug lgo");
    let (ui_send, model_recv) = channel();
    let (model_send, ui_recv) = channel();

    let _ = thread::spawn(move || {
        init_model(ui_send, ui_recv);
    });
    ui::init_ui(model_send, model_recv)?;
    Ok(())
}

fn test_init() {
    Logger::try_with_str("debug")
        .unwrap()
        .log_to_stdout()
        .start()
        .unwrap();
}
