use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use rouille::{Response, router};
use serde::Serialize;
use speedy2d::window::UserEventSender;

use crate::window_handler::SignWindowHandler;

#[derive(Serialize)]
struct StatusResponse {
    root_path: PathBuf,
    is_fullscreen: bool,
}

pub fn start_server(handler: &SignWindowHandler, sender: Mutex<UserEventSender<String>>) {
    let path = handler.root_path.clone();
    let is_fullscreen = handler.is_fullscreen.clone();
    
    thread::spawn(move || {          
        rouille::start_server("127.0.0.1:3000", move |request| {         
            let response = rouille::match_assets(&request, "frontend/dist");
            if response.is_success() {
                return response;
            }
            
            router!(request, 
                (GET) (/api/status) => {
                    let data = StatusResponse {
                      root_path: path.lock().unwrap().to_owned(),
                      is_fullscreen: *is_fullscreen.lock().unwrap(),
                    };
                    
                    Response::json(&data)
                },
                (GET) (/api/test_sender) => {
                    sender.lock().unwrap().send_event("Test".to_owned()).unwrap();
                    Response::text("Sending test")
                },
                _ => {
                    Response::text("Unknown route!")
                },
            )
        });
    });
}