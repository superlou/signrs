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
use speedy2d::shape::Rectangle;
use speedy2d::window::{
    WindowHandler, WindowHelper, WindowStartupInfo,
    MouseButton, WindowFullscreenMode
};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::font::FormattedTextBlock;
use speedy2d::dimen::Vec2;
use thiserror::Error;

use crate::iter_util::iter_unique;
use crate::js_env::JsEnv;
use crate::js_draw;

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
    graphics_calls: Rc<RefCell<Vec<GraphicsCalls>>>,
    draw_offset_stack: Vec<Vec2>,
    draw_offset: Vec2,
    pub root_path: Arc<Mutex<PathBuf>>,
    image_handles: Rc<RefCell<HashMap<String, ImageHandle>>>,
    #[allow(deprecated)]
    watches: Rc<RefCell<HashMap<PathBuf, JsFunction>>>,
    
    #[allow(dead_code)] // Required to keep watcher in scope
    watcher: Box<dyn Watcher>,
    
    file_change_rx: mpsc::Receiver<PathBuf>,
    file_change_tx: mpsc::Sender<PathBuf>,
}

pub enum GraphicsCalls {
    ClearScreenBlack,
    ClearScreen(Color),
    DrawRectangle(Rectangle, Color),
    DrawText(Vec2, Color, Rc<FormattedTextBlock>),
    DrawImage(Vec2, String),
    DrawRectangleImageTinted(Rectangle, String, Color),
    PushOffset(Vec2),
    PopOffset(),
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
                match JsEnv::load_json(&changed_path_buf, &mut self.script_env.context) {
                    Ok(data) => {
                        if let Err(err) = js_fn.call(&JsValue::Undefined, &[data], &mut self.script_env.context) {
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
            let mut script_env = JsEnv::new(&root_path);
            
            js_draw::register_fns_and_types(
                &mut script_env,
                &self.graphics_calls,
                &self.watches,
            );
            
            match script_env.call_init() {
                Ok(_) => {
                    self.script_env = script_env;
                    println!("Reloaded script environment.");
                },
                Err(err) => { dbg!(&err); },
            };
        }
        
        // Call script draw function
        if let Err(err) = self.script_env.call_draw(dt) {
            println!("{}", err);
        }

        // Perform queued graphic calls
        for call in self.graphics_calls.clone().borrow().iter() {
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
        self.graphics_calls.borrow_mut().clear();
        
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
             
        let mut handler = SignWindowHandler {
            script_env: JsEnv::new(app_root.as_ref()),
            last_frame_time: Instant::now(),
            last_mouse_down_time: None,
            is_fullscreen: Arc::new(Mutex::new(false)),
            graphics_calls: Rc::new(RefCell::new(vec![])),
            draw_offset: Vec2::ZERO,
            draw_offset_stack: vec![],
            root_path: Arc::new(Mutex::new(app_root.as_ref().to_path_buf())),
            image_handles: Rc::new(RefCell::new(HashMap::new())),
            watches: Rc::new(RefCell::new(HashMap::new())),
            watcher: Box::new(watcher),
            file_change_tx: tx,
            file_change_rx: rx,
        };
        
        js_draw::register_fns_and_types(
            &mut handler.script_env,
            &handler.graphics_calls,
            // &handler.root_path,
            &handler.watches,
        );
        
        if let Err(err) = handler.script_env.call_init() {
            dbg!(err);
            panic!("Unable to initialize script environment!");
        }

        handler
    }
    
    pub fn get_resolution(&self) -> Option<(u32, u32)> {
        // match self.script_env.get_state_value("resolution").unwrap().into_array() {
        //     Ok(a) => {
        //         match (a[0].clone().try_cast::<i64>(), a[1].clone().try_cast::<i64>()) {
        //             (Some(x), Some(y)) => Some((x as u32, y as u32)),
        //             _ => None,
        //         }
        //     },
        //     Err(_) => None
        // }
        
        Some((640, 480))
    }
    
    pub fn get_multisampling(&self) -> Option<u16> {
        // match self.script_env.get_state_value("multisampling").unwrap().as_int() {
        //     Ok(m) => u16::try_from(m).ok(),
        //     Err(_) => None,
        // }
        
        Some(1)
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