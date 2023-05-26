use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

use boa_engine::JsValue;
use boa_engine::object::builtins::JsFunction;
use notify::{Watcher, RecursiveMode};
use speedy2d::image::{ImageHandle, ImageSmoothingMode};
use speedy2d::window::{
    WindowHandler, WindowHelper, WindowStartupInfo,
    MouseButton, WindowFullscreenMode
};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::dimen::Vec2;
use thiserror::Error;

use crate::iter_util::iter_unique;
use crate::js_env::JsEnv;
use crate::js_draw::GraphicsCalls;

#[derive(Error, Debug)]
enum SignError {
    // #[error("EvalAltError: {0}")]
    // EvalAltError(#[from] ScriptError),
}

// #[derive(Clone)]
// enum UserEvents {
//     Index,
// }

pub struct SignWindowHandler {
    script_env: JsEnv,
    last_frame_time: Instant,
    last_mouse_down_time: Option<Instant>,
    pub is_fullscreen: Arc<Mutex<bool>>,
    draw_offset_stack: Vec<Vec2>,
    draw_offset: Vec2,
    pub root_path: Arc<Mutex<PathBuf>>,
    image_handles: Rc<RefCell<HashMap<String, ImageHandle>>>,
    #[allow(deprecated)]
    watches: Rc<RefCell<HashMap<PathBuf, JsFunction>>>,
    
    #[allow(dead_code)] // Required to keep watcher in scope
    watcher: Box<dyn Watcher>,
    
    file_change_rx: mpsc::Receiver<PathBuf>,
    _file_change_tx: mpsc::Sender<PathBuf>,
}

impl WindowHandler<String> for SignWindowHandler {
    fn on_start(&mut self, helper: &mut WindowHelper<String>, _info: WindowStartupInfo) {
        let sender = helper.create_user_event_sender();
        crate::server::start_server(&self, Mutex::new(sender));
    }
    
    fn on_draw(&mut self, helper: &mut WindowHelper<String>, graphics: &mut Graphics2D) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();
        
        let mut reload_script_env = false;
        
        for changed_path_buf in iter_unique(self.file_change_rx.try_iter()) {
            // Check if it's a watched file with a callback
            if let Some(js_fn) = self.watches.borrow().get(&changed_path_buf) {               
                match JsEnv::load_json(&changed_path_buf, self.script_env.context_mut()) {
                    Ok(data) => {
                        if let Err(err) = js_fn.call(&JsValue::Undefined, &[data], self.script_env.context_mut()) {
                            dbg!(&err);
                        }
                    },
                    Err(err) => {dbg!(&err);},
                }
            }
            
            // If not explicitly watched, do other updates
            let extension = changed_path_buf.extension().and_then(|ext| ext.to_str());            
            match extension {
                Some(ext) if ext == "js" => {
                    reload_script_env = true;
                },
                _ => {},
            }
        }
        
        if reload_script_env {
            let root_path = self.root_path.lock().unwrap().clone();
            match JsEnv::new(&root_path, &self.watches) {
                Ok(mut script_env) => match script_env.call_init() {
                    Ok(_) => {
                        self.script_env = script_env;
                        println!("Reloaded script environment.");
                    },
                    Err(err) => { dbg!(&err); },
                },
                Err(err) => { dbg!(&err); },
            }
        }
        
        // Call script draw function
        if let Err(err) = self.script_env.call_draw(dt) {
            println!("{}", err);
        }

        // Perform queued graphic calls
        for call in self.script_env.graphics_calls().clone().borrow().iter() {
            match call {
                GraphicsCalls::ClearScreenBlack => {
                  graphics.clear_screen(Color::BLACK);  
                },
                GraphicsCalls::ClearScreen(c) => graphics.clear_screen(*c),
                GraphicsCalls::DrawRectangle(r, c) => {
                    graphics.draw_rectangle(r.with_offset(self.draw_offset), *c)
                },
                GraphicsCalls::DrawRectangleImageTinted(r, path_string, c) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    graphics.draw_rectangle_image_tinted(
                        r.with_offset(self.draw_offset),
                        *c,
                        &image_handle
                    );
                },
                GraphicsCalls::DrawText(pos, c, block) => {
                    graphics.draw_text(pos + self.draw_offset, *c, block)
                },
                GraphicsCalls::DrawImage(pos, path_string) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    graphics.draw_image(pos + self.draw_offset, &image_handle);
                },
                GraphicsCalls::PushOffset(vec2) => {
                    self.draw_offset += *vec2;
                    self.draw_offset_stack.push(*vec2);
                }
                GraphicsCalls::PopOffset() => {
                    self.draw_offset -= self.draw_offset_stack.pop().unwrap_or(Vec2::ZERO);
                }
            }
        }
        self.script_env.clear_graphics_calls();
        
        helper.request_redraw();
    }
    
    fn on_mouse_button_down(&mut self, helper: &mut WindowHelper<String>, _button: MouseButton) {
        let double_click_timeout = Duration::from_millis(500);
        let now = Instant::now();
        
        if let Some(prev_down) = self.last_mouse_down_time {
            if now - prev_down < double_click_timeout {
                self.toggle_fullscreen(helper);
            }
        }
        
        self.last_mouse_down_time = Some(now);
    }
    
    fn on_fullscreen_status_changed(&mut self, _helper: &mut WindowHelper<String>, fullscreen: bool) {
        *self.is_fullscreen.lock().unwrap() = fullscreen;
    }
    
    fn on_user_event(
        &mut self,
        _helper: &mut WindowHelper<String>,
        user_event: String
    ) {
        println!("{}", user_event);
    }
}

impl SignWindowHandler {
    fn toggle_fullscreen(&mut self, helper: &mut WindowHelper<String>) {
        if *self.is_fullscreen.lock().unwrap() {
            helper.set_fullscreen_mode(WindowFullscreenMode::Windowed);
        } else {
            helper.set_fullscreen_mode(WindowFullscreenMode::FullscreenBorderless);
        }
    }
    
    pub fn new<P: AsRef<Path>>(app_root: P) -> Self {       
        let (tx, rx) = mpsc::channel();
        let tx_for_watcher = tx.clone();
         
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(event) => match event.kind {
                    notify::EventKind::Modify(_) => {
                      for path_buf in event.paths {
                          let _ = tx_for_watcher.send(path_buf);
                      }
                    },
                    _ => (),
                },
                Err(err) => println!("Watch error: {:?}", err),
            }
        }).unwrap();
         
        watcher.watch(app_root.as_ref(), RecursiveMode::Recursive).unwrap();
             
        let watches = Rc::new(RefCell::new(HashMap::new()));
             
        let mut handler = SignWindowHandler {
            script_env: JsEnv::new(app_root.as_ref(), &watches).unwrap(),
            last_frame_time: Instant::now(),
            last_mouse_down_time: None,
            is_fullscreen: Arc::new(Mutex::new(false)),
            draw_offset: Vec2::ZERO,
            draw_offset_stack: vec![],
            root_path: Arc::new(Mutex::new(app_root.as_ref().to_path_buf())),
            image_handles: Rc::new(RefCell::new(HashMap::new())),
            watches,
            watcher: Box::new(watcher),
            _file_change_tx: tx,
            file_change_rx: rx,
        };
        
        if let Err(err) = handler.script_env.call_init() {
            dbg!(err);
            panic!("Unable to initialize script environment!");
        }

        handler
    }
    
    pub fn get_resolution(&mut self) -> Option<(u32, u32)> {
        match self.script_env.get_array::<u32, _>("resolution") {
            Ok(a) if a.len() >= 2 => Some((a[0], a[1])),
            _ => None,
        }
    }
    
    pub fn get_multisampling(&mut self) -> Option<u16> {
        match self.script_env.get_value::<i32, _>("multisampling") {
            Ok(value) => Some(value as u16),
            Err(_) => None,
        }
    }
    
    fn get_image_handle(&mut self, path_string: &str, graphics: &mut Graphics2D) -> ImageHandle {
        let mut created = false;
        let image_handle = match self.image_handles.borrow_mut().get_mut(path_string) {
            Some(image_handle) => image_handle.clone(),
            None => {
                let mut path = self.root_path.lock().unwrap().clone();
                path.push(path_string);
                let image_handle = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear, path).unwrap();
                created = true;
                image_handle
            }
        };
                    
        if created {
            self.image_handles.borrow_mut().insert(path_string.to_owned(), image_handle.clone());
        }
        
        image_handle
    }
}