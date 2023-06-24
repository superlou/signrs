use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, Duration};

use speedy2d::image::{ImageHandle, ImageSmoothingMode};
use speedy2d::window::{
    WindowHandler, WindowHelper, WindowStartupInfo,
    MouseButton, WindowFullscreenMode
};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::dimen::Vec2;
use thiserror::Error;

use crate::js_env::{JsEnv, GraphicsCalls};

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
}

impl WindowHandler<String> for SignWindowHandler {
    fn on_start(&mut self, helper: &mut WindowHelper<String>, _info: WindowStartupInfo) {
        let sender = helper.create_user_event_sender();
        crate::server::start_server(self, Mutex::new(sender));
    }
    
    fn on_draw(&mut self, helper: &mut WindowHelper<String>, graphics: &mut Graphics2D) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();
             
        self.script_env.handle_file_changes();
        
        // Call script draw function
        if let Err(err) = self.script_env.call_draw(dt) {
            println!("{}", err);
        }

        // Perform queued graphic calls
        self.draw_offset_stack.clear();
        self.draw_offset = (0., 0.).into();
        
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
                    graphics.draw_text(pos + self.draw_offset, *c, block);
                },
                GraphicsCalls::DrawImage(pos, path_string) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    graphics.draw_image(pos + self.draw_offset, &image_handle);
                },
                GraphicsCalls::PushOffset(vec2) => {
                    self.draw_offset += *vec2;
                    self.draw_offset_stack.push(*vec2);
                }
                GraphicsCalls::PopOffset => {
                    self.draw_offset -= self.draw_offset_stack.pop().unwrap_or(Vec2::ZERO);
                },
                GraphicsCalls::SetResolution(uvec2) => {
                    graphics.set_resolution(*uvec2);
                    helper.set_size_pixels(uvec2);
                }
            }
        }
        
        // Clear graphics calls only once we've handled them, including calls from
        // initialization.
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
        let mut script_env = JsEnv::new(app_root.as_ref());
        if let Err(err) = script_env.call_init() {
            dbg!(err);
        }
        
        let mut arc_graphics_calls: Arc<RwLock<Vec<GraphicsCalls>>> = Arc::new(RwLock::new(vec![]));
        let mut arc_graphics_calls_ = arc_graphics_calls.clone();
        let app_root_ = app_root.as_ref().to_owned();
        std::thread::spawn(move || {
            let mut script_env = JsEnv::new(&app_root_);
            if let Err(err) = script_env.call_init() {
                dbg!(err);
            }
            
            loop {
                script_env.handle_file_changes();
                if let Err(err) = script_env.call_draw(1. / 60.) {
                    dbg!(err);
                }
                
                let graphics_calls = script_env.graphics_calls();
                let mut arcgc = arc_graphics_calls_.write().unwrap();
                arcgc.append(&mut graphics_calls.borrow_mut());
                script_env.clear_graphics_calls()
            } 
        });
        
        std::thread::sleep(Duration::from_millis(100));
        dbg!(arc_graphics_calls.read().unwrap().len());
        std::thread::sleep(Duration::from_millis(100));
        dbg!(arc_graphics_calls.read().unwrap().len());
        std::thread::sleep(Duration::from_millis(100));
        dbg!(arc_graphics_calls.read().unwrap());                                      
                                        
        SignWindowHandler {
            script_env,
            last_frame_time: Instant::now(),
            last_mouse_down_time: None,
            is_fullscreen: Arc::new(Mutex::new(false)),
            draw_offset: Vec2::ZERO,
            draw_offset_stack: vec![],
            root_path: Arc::new(Mutex::new(app_root.as_ref().to_path_buf())),
            image_handles: Rc::new(RefCell::new(HashMap::new())),
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