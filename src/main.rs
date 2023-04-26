use speedy2d::Window;
use speedy2d::window::{WindowCreationOptions, WindowSize};

mod server;
mod iter_util;
mod script_env;
mod window_handler;
use window_handler::SignWindowHandler;


fn main() {
    println!("Starting...");
    let handler = SignWindowHandler::new("examples/app1");
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