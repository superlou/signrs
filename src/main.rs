use speedy2d::Window;
use speedy2d::window::{WindowCreationOptions, WindowSize};

mod server;
mod iter_util;
mod window_handler;
mod js_env;
mod perf;
use window_handler::SignWindowHandler;

const HELP: &str = "\
signrs digital signage application player

USAGE:
  signrs [APPLICATION]

FLAGS:
  -h, --help       Prints help information

OPTIONS:
  --multisampling  Sets the multisampling level [default: 1]
  -p, --port       Sets the server port [default: 3000]
";

#[derive(Debug)]
struct SignArgs {
    app_path: String,
    multisampling: u16,
    port: u16,
}

fn parse_args() -> Result<SignArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    
    let args = SignArgs {
        app_path: pargs.free_from_str()?,
        multisampling: pargs.opt_value_from_str("--multisampling")?.unwrap_or(1),
        port: pargs.opt_value_from_str(["-p", "--port"])?.unwrap_or(3000),
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
    let handler = SignWindowHandler::new(&app_path, args.port);

    let options = WindowCreationOptions::new_windowed(WindowSize::PhysicalPixels((640, 480).into()), None)
                    .with_multisampling(args.multisampling)
                    .with_fixed_resolution(true);
    
    let window: Window<String> = Window::new_with_user_events("Title", options)
        .expect("Failed to create window!");
    
    println!("Starting {}...", &app_path);
    println!("Multisampling: {}", args.multisampling);
    window.run_loop(handler);    
    
}