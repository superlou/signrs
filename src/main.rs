use speedy2d::Window;

mod window_handler;
use window_handler::SignWindowHandler;

fn main() {
    println!("Starting...");
    let handler = SignWindowHandler::new("examples/display1");
    let resolution = handler.get_resolution().expect("Script didn't set resolution!");    
    let window = Window::new_centered("Title", resolution).unwrap();    
    window.run_loop(handler);
}
