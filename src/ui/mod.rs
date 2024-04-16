use std::{
    default,
    sync::mpsc::{Receiver, Sender},
};

use eframe::egui;
use log::{error, info};

use crate::control::{ModelMsg, UiMsg};

struct EdenChessUI {
    model_sender: Sender<UiMsg>,
    model_reciever: Receiver<ModelMsg>,
}
impl EdenChessUI {
    fn new(
        cc: &eframe::CreationContext<'_>,
        send: Sender<UiMsg>,
        recv: Receiver<ModelMsg>,
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
                if let Err(e) = self.model_sender.send(UiMsg::Debug("Clicked!")) {
                    error!("{}", e);
                };
            }
        });
    }
}

pub fn init_ui(send: Sender<UiMsg>, recv: Receiver<ModelMsg>) {
    let native_options = eframe::NativeOptions::default();
    info!("Glog");
    eframe::run_native(
        "Eden Chess",
        native_options,
        Box::new(|cc| Box::new(EdenChessUI::new(cc, send, recv))),
    );
}
