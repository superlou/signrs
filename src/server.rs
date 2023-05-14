use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

use rouille::{Response, router};
use serde::Serialize;
use speedy2d::window::UserEventSender;
use walkdir::WalkDir;

use crate::window_handler::SignWindowHandler;

#[derive(Serialize)]
struct StatusResponse {
    root_path: PathBuf,
    is_fullscreen: bool,
}

trait ResponseHelpers {
    fn allow_cors(self) -> Self;
}

impl ResponseHelpers for Response {
    fn allow_cors(self) -> Self {
        self.with_additional_header("Access-Control-Allow-Origin", "*")
    }
}

fn get_dir_contents(path: impl AsRef<Path>) -> Vec<DirItem> {
    WalkDir::new(path)
        .into_iter()
        .map(|entry| {
            let entry = entry.unwrap();
            
            DirItem {
                name: entry.path().to_str().unwrap().to_owned(),
                is_dir: entry.path().is_dir(),
            }
        })
        .collect()
}

#[derive(Serialize)]
struct DirItem {
    name: String,
    is_dir: bool,
}

#[derive(Serialize)]
struct DirData {
    kind: String,
    items: Vec<DirItem>,
}

#[derive(Serialize)]
struct FileData {
    kind: String, 
    content: String,
}

#[derive(Serialize)]
struct PathNotFound {
    kind: String,
}

fn make_fs_response(path: &Path) -> Response {
    if path.is_dir() {
        Response::json(&DirData {
            kind: "dir".to_owned(),
            items: get_dir_contents(path),
        })
    } else if path.is_file() {
        Response::json(&FileData {
            kind: "file".to_owned(),
            content: read_to_string(path).unwrap_or("".to_owned()),
        })
    } else {
        Response::json(&PathNotFound {
            kind: "unknown".to_owned(),
        })
    }
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
            
            if let Some(request) = request.remove_prefix("/api/fs/") {
                let mut path = path.lock().unwrap().to_owned();
                path.push(request.url());
                return make_fs_response(&path).allow_cors();
            }
            
            let response = router!(request, 
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
            );
            
            response.allow_cors()
        });
    });
}