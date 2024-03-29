use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex, RwLock, mpsc, atomic};
use std::thread;
use std::time::{Instant, Duration};

use speedy2d::image::{ImageHandle, ImageSmoothingMode};
use speedy2d::window::{
    WindowHandler, WindowHelper, WindowStartupInfo,
    MouseButton, WindowFullscreenMode
};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::dimen::Vec2;
use speedy2d::numeric::RoundFloat;
use thiserror::Error;

use crate::js_env::{JsEnv, GraphicsCalls};
use crate::perf::Perf;

#[derive(Error, Debug)]
enum SignError {
    // #[error("EvalAltError: {0}")]
    // EvalAltError(#[from] ScriptError),
}

enum JsThreadMsg {
    RunFrame(f32),
    TerminateThread,
}

// #[derive(Clone)]
// enum UserEvents {
//     Index,
// }

pub struct SignWindowHandler {
    graphics_calls: Arc<RwLock<Vec<GraphicsCalls>>>,
    js_thread_tx: Sender<JsThreadMsg>,
    last_frame_time: Instant,
    last_mouse_down_time: Option<Instant>,
    pub is_fullscreen: Arc<Mutex<bool>>,
    draw_offset_stack: Vec<Vec2>,
    draw_offset: Vec2,
    pub root_path: Arc<Mutex<PathBuf>>,
    image_handles: Rc<RefCell<HashMap<String, ImageHandle>>>,
    draw_perf: Perf,
    server_port: u16,
}

impl WindowHandler<String> for SignWindowHandler {
    fn on_start(&mut self, helper: &mut WindowHelper<String>, _info: WindowStartupInfo) {
        let sender = helper.create_user_event_sender();
        crate::server::start_server(self, Mutex::new(sender), self.server_port);
    }
    
    fn on_draw(&mut self, helper: &mut WindowHelper<String>, graphics: &mut Graphics2D) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();

        // Wait for graphics_calls access when the JsEnv thread finishes
        //let graphics_calls = self.graphics_calls.read().unwrap().clone();
        let graphics_calls = std::mem::take(&mut *self.graphics_calls.write().unwrap());
        self.js_thread_tx.send(JsThreadMsg::RunFrame(dt)).unwrap();        
        
        self.draw_perf.start();
        
        // Perform queued graphic calls
        self.draw_offset_stack.clear();
        self.draw_offset = (0., 0.).into();
        
        for call in graphics_calls.iter() {
            use GraphicsCalls::*;
            match call {
                ClearScreenBlack => graphics.clear_screen(Color::BLACK),
                ClearScreen(c) => graphics.clear_screen(*c),
                DrawRectangle(r, c) => {
                    graphics.draw_rectangle(r.with_offset(self.draw_offset), *c)
                },
                DrawRectangleImageTinted(r, path_string, c) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    graphics.draw_rectangle_image_tinted(
                        r.with_offset(self.draw_offset),
                        *c,
                        &image_handle
                    );
                },
                DrawText(pos, c, block) => {
                    // Rounding position avoids subpixel positions to improve performance
                    let relative_pos = (pos + self.draw_offset).round();
                    graphics.draw_text(relative_pos, *c, block);
                },
                DrawImage(pos, path_string) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    let relative_pos = pos + self.draw_offset;
                    graphics.draw_image(relative_pos, &image_handle);
                },
                PushOffset(vec2) => {
                    self.draw_offset += *vec2;
                    self.draw_offset_stack.push(*vec2);
                }
                PopOffset => {
                    self.draw_offset -= self.draw_offset_stack.pop().unwrap_or(Vec2::ZERO);
                },
                SetResolution(uvec2) => {
                    graphics.set_resolution(*uvec2);
                    helper.set_size_pixels(uvec2);
                },
                ImageFileUpdate(pathbuf) => {
                    self.update_image_handle(pathbuf, graphics)
                }
            }
        }
        
        self.draw_perf.stop();
        self.draw_perf.report_after(Duration::from_secs(1));

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

impl Drop for SignWindowHandler {
    fn drop(&mut self) {
        self.js_thread_tx.send(JsThreadMsg::TerminateThread).unwrap();
    }
}

fn js_thread(app_root: PathBuf, ready: Arc<AtomicBool>,
    arc_graphics_calls: Arc<RwLock<Vec<GraphicsCalls>>>,
    js_thread_rx: Receiver<JsThreadMsg>
) {
    thread::spawn(move || {
        let mut js_frame_perf = Perf::new("JS frame");
        
        let mut script_env = JsEnv::new(&app_root);
        if let Err(err) = script_env.call_init() {
            dbg!(err);
        }
        
        ready.store(true, atomic::Ordering::SeqCst);
        
        loop {
            match js_thread_rx.recv().expect("No remaining senders!") {
                JsThreadMsg::RunFrame(dt) => {
                    js_frame_perf.start();
                    // Immediately hold the RwLock so the drawing thread has to wait
                    let mut arcgc = arc_graphics_calls.write().unwrap();
    
                    script_env.handle_file_changes();
                    if let Err(err) = script_env.call_draw(dt) {
                        dbg!(err);
                    }
                    
                    let graphics_calls = script_env.graphics_calls();
                    arcgc.clear();
                    arcgc.append(&mut graphics_calls.borrow_mut());
                    script_env.clear_graphics_calls();
                    js_frame_perf.stop();
                    js_frame_perf.report_after(Duration::from_secs(1));
                },
                JsThreadMsg::TerminateThread => return,
            }
        } 
    });
}

impl SignWindowHandler {
    fn toggle_fullscreen(&mut self, helper: &mut WindowHelper<String>) {
        if *self.is_fullscreen.lock().unwrap() {
            helper.set_fullscreen_mode(WindowFullscreenMode::Windowed);
        } else {
            helper.set_fullscreen_mode(WindowFullscreenMode::FullscreenBorderless);
        }
    }
    
    pub fn new<P: AsRef<Path>>(app_root: P, server_port: u16) -> Self {       
        let (js_thread_tx, js_thread_rx) = mpsc::channel();
        let arc_graphics_calls: Arc<RwLock<Vec<GraphicsCalls>>> = Arc::new(RwLock::new(vec![]));
        let js_ready = Arc::new(AtomicBool::new(false));
        
        js_thread(
            app_root.as_ref().to_owned(),
            js_ready.clone(),
            arc_graphics_calls.clone(),
            js_thread_rx
        );
        
        print!("Waiting for JS environment to start...");
        while !js_ready.load(atomic::Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
        println!("done.");

        SignWindowHandler {
            graphics_calls: arc_graphics_calls,
            js_thread_tx,
            last_frame_time: Instant::now(),
            last_mouse_down_time: None,
            is_fullscreen: Arc::new(Mutex::new(false)),
            draw_offset: Vec2::ZERO,
            draw_offset_stack: vec![],
            root_path: Arc::new(Mutex::new(app_root.as_ref().to_path_buf())),
            image_handles: Rc::new(RefCell::new(HashMap::new())),
            draw_perf: Perf::new("Graphics draw"),
            server_port,
        }
    }
    
    fn get_image_handle(&mut self, path_string: &str, graphics: &mut Graphics2D) -> ImageHandle {               
        if let Some(image_handle) = self.image_handles.borrow_mut().get_mut(path_string) {
            return image_handle.clone();
        }

        // The path_string wasn't found in the image_handles map, so we need to create it.
        let mut path = self.root_path.lock().unwrap().clone();
        path.push(path_string);
        let image_handle = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear, path).unwrap();
        self.image_handles.borrow_mut().insert(path_string.to_owned(), image_handle.clone());
        image_handle
    }

    fn update_image_handle(&mut self, path: &Path, graphics: &mut Graphics2D) {
        let image_handle = graphics.create_image_from_file_path(
            None, ImageSmoothingMode::Linear, path
        ).unwrap();
        let root_path = self.root_path.lock().unwrap().clone();
        let key = path.strip_prefix(root_path).unwrap()
            .to_str().unwrap().to_owned();
        self.image_handles.borrow_mut().insert(key, image_handle);
    }
}