use speedy2d::Window;
use speedy2d::window::{WindowCreationOptions, WindowSize};

mod iter_util;
mod window_handler;
use window_handler::SignWindowHandler;

fn main() {
    println!("Starting...");
    let handler = SignWindowHandler::new("examples/display1");
    let resolution = handler.get_resolution().expect("Script didn't set resolution!");
    let multisampling = handler.get_multisampling().unwrap_or(1_u16);
    
    println!("Resolution: {}x{}", resolution.0, resolution.1);
    println!("Multisampling: {}", multisampling);

    let options = WindowCreationOptions::new_windowed(WindowSize::PhysicalPixels(resolution.into()), None)
                    .with_multisampling(multisampling);
    
    let window = Window::new_with_options("Title", options).expect("Failed to create window!");

    window.run_loop(handler);
}
