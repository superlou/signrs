use std::sync::Mutex;
use std::thread;

use rouille::{Request, Response};
use speedy2d::window::UserEventSender;

use crate::window_handler::SignWindowHandler;

pub fn start_server(handler: &SignWindowHandler, sender: Mutex<UserEventSender<String>>) {
    let path = handler.root_path.clone();
    
    thread::spawn(move || {          
        rouille::start_server("0.0.0.0:3000", move |_request| {
            sender.lock().unwrap().send_event("Test".to_owned()).unwrap();
            let path_str = path.lock().unwrap();
            Response::text(path_str.to_str().unwrap())
        });
    });
}