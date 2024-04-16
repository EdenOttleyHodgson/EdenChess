use std::{
    default,
    sync::mpsc::{Receiver, Sender},
};

use eframe::egui;
use log::{error, info};

use crate::control::ControlMsg;

struct EdenChessUI {
    model_sender: Sender<ControlMsg>,
    model_reciever: Receiver<ControlMsg>,
}
impl EdenChessUI {
    fn new(
        cc: &eframe::CreationContext<'_>,
        send: Sender<ControlMsg>,
        recv: Receiver<ControlMsg>,
    ) -> Self {
        EdenChessUI {
            model_sender: send,
            model_reciever: recv,
        }
    }
}
impl eframe::App for EdenChessUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello world!");
            if ui.button("Click Me").clicked() {
                if let Err(e) = self.model_sender.send(ControlMsg::Debug) {
                    error!("{}", e);
                };
            }
        });
    }
}

pub fn init_ui(send: Sender<ControlMsg>, recv: Receiver<ControlMsg>) {
    let native_options = eframe::NativeOptions::default();
    info!("Glog");
    eframe::run_native(
        "Eden Chess",
        native_options,
        Box::new(|cc| Box::new(EdenChessUI::new(cc, send, recv))),
    );
}
