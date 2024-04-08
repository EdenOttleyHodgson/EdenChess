use std::sync::mpsc::{Receiver, Sender};

use log::{error, info};

use crate::ControlMsg;

struct Game {
    ui_sender: Sender<ControlMsg>,
    ui_reciever: Receiver<ControlMsg>,
}
impl Game {
    fn new(send: Sender<ControlMsg>, recv: Receiver<ControlMsg>) -> Self {
        Game {
            ui_sender: send,
            ui_reciever: recv,
        }
    }
}

pub fn init_model(send: Sender<ControlMsg>, recv: Receiver<ControlMsg>) {
    let game = Game::new(send, recv);
    if let Err(e) = game.ui_sender.send(ControlMsg::Debug) {
        error!("{}", e)
    };
    while let Ok(msg) = game.ui_reciever.recv() {
        info!("Msg recieved: {}", msg)
    }
}
