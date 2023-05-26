use std::env;

use speedy2d::Window;
use speedy2d::window::{WindowCreationOptions, WindowSize};

mod server;
mod iter_util;
mod window_handler;
mod js_env;
mod js_draw;
use window_handler::SignWindowHandler;

fn main() { 
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("No application specified!");
        return;
    }
    
    let app_path = &args[1];
    println!("Starting {}...", &app_path);
    let mut handler = SignWindowHandler::new(app_path);
    let resolution = handler.get_resolution().expect("Script didn't set resolution!");
    let multisampling = handler.get_multisampling().unwrap_or(1_u16);
    
    println!("Resolution: {}x{}", resolution.0, resolution.1);
    println!("Multisampling: {}", multisampling);

    let options = WindowCreationOptions::new_windowed(WindowSize::PhysicalPixels(resolution.into()), None)
                    .with_multisampling(multisampling).with_stretch(true);
    
    let window: Window<String> = Window::new_with_user_events("Title", options)
        .expect("Failed to create window!");
    
    window.run_loop(handler);    
    
}