use speedy2d::Window;
use speedy2d::window::{WindowCreationOptions, WindowSize};

mod server;
mod iter_util;
mod window_handler;
mod js_env;
mod js_draw;
use window_handler::SignWindowHandler;

const HELP: &str = "\
signrs digital signage application player

USAGE:
  signrs [APPLICATION]

FLAGS:
  -h, --help       Prints help information
";

#[derive(Debug)]
struct SignArgs {
    app_path: String,
}

fn parse_args() -> Result<SignArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    
    let args = SignArgs {
        app_path: pargs.free_from_str()?,
    };
    
    Ok(args)
}

fn main() { 
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    
    let app_path = args.app_path;
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