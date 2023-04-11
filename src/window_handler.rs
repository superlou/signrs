use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use speedy2d::image::{ImageHandle, ImageSmoothingMode};
use speedy2d::shape::Rectangle;
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::font::{Font, TextLayout, TextOptions, FormattedTextBlock};
use speedy2d::dimen::Vec2;
use rhai::{Engine, Scope, AST, Array, CallFnOptions};

pub struct SignWindowHandler {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
    last_frame_time: Instant,
    graphics_calls: Rc<RefCell<Vec<GraphicsCalls>>>,
    root_path: Rc<RefCell<PathBuf>>,
    image_handles: Rc<RefCell<HashMap<String, ImageHandle>>>,
}

enum GraphicsCalls {
    ClearScreen(Color),
    DrawRectangle(Rectangle, Color),
    DrawText(Vec2, Color, Rc<FormattedTextBlock>),
    DrawImage(Vec2, String),
    DrawRectangleImageTinted(Rectangle, String, Color),
}

impl WindowHandler for SignWindowHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();
      
        let options = CallFnOptions::new().eval_ast(false);
        let result = self.engine.call_fn_with_options::<()>(options, &mut self.scope, &mut self.ast, "draw", (dt,));
        if let Err(err) = result {
            dbg!(&err);
        }

        for call in self.graphics_calls.clone().borrow().iter() {
            match call {
                GraphicsCalls::ClearScreen(c) => graphics.clear_screen(*c),
                GraphicsCalls::DrawRectangle(r, c) => graphics.draw_rectangle(r.clone(), *c),
                GraphicsCalls::DrawText(pos, c, block) => graphics.draw_text(pos, *c, block),
                GraphicsCalls::DrawImage(pos, path_string) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    graphics.draw_image(pos, &image_handle);
                },
                GraphicsCalls::DrawRectangleImageTinted(r, path_string, c) => {
                    let image_handle = self.get_image_handle(path_string, graphics);
                    graphics.draw_rectangle_image_tinted(r.clone(), *c, &image_handle);
                },
            }
        }
        self.graphics_calls.borrow_mut().clear();
        
        helper.request_redraw();
    }
}

impl SignWindowHandler {
    pub fn new<P: AsRef<Path>>(sign_root: P) -> Self {
        let engine = Engine::new();   
        
        let mut main_script = PathBuf::from(sign_root.as_ref());
        main_script.push("main.rhai");
        let ast = engine.compile_file(main_script).unwrap();        
        
        let mut handler = SignWindowHandler {
            engine, ast,
            scope: Scope::new(),
            last_frame_time: Instant::now(),
            graphics_calls: Rc::new(RefCell::new(vec![])),
            root_path: Rc::new(RefCell::new(sign_root.as_ref().to_path_buf())),
            image_handles: Rc::new(RefCell::new(HashMap::new()))
        };
        
        handler.setup_engine();
        handler
    }

    fn setup_engine(&mut self) {
        let graphics_calls = self.graphics_calls.clone();       
        self.engine.register_fn("clear_screen", move |c: Color| {
           graphics_calls.borrow_mut().push(GraphicsCalls::ClearScreen(c));
        });
        
        let graphics_calls = self.graphics_calls.clone();        
        self.engine.register_fn("draw_rectangle", move |ulx: f32, uly: f32, llx: f32, lly: f32, c: Color| {
           let r = Rectangle::from_tuples((ulx, uly), (llx, lly));         
           graphics_calls.borrow_mut().push(GraphicsCalls::DrawRectangle(r, c));
        });
        
        let graphics_calls = self.graphics_calls.clone();
        self.engine.register_fn("draw_text", move |text: &str, font: Font, scale: f32, color: Color, x: f32, y: f32| {
            let block = font.layout_text(text, scale, TextOptions::new());
            graphics_calls.borrow_mut().push(GraphicsCalls::DrawText((x, y).into(), color, block));
        });
    
        self.engine.register_type_with_name::<Color>("Color")
            .register_fn("new_color_from_rgb", Color::from_rgb);

        self.engine.register_type_with_name::<Color>("Color")
            .register_fn("new_color_from_rgba", Color::from_rgba);
        
        let root_path = self.root_path.clone();
        self.engine.register_type_with_name::<Font>("Font")
            .register_fn("new_font", move |font_path: &str| {
                let mut full_path = root_path.borrow().clone();
                full_path.push(font_path);
                dbg!(&full_path);
                
                let bytes = std::fs::read(full_path.as_path()).unwrap();
                let font = Font::new(&bytes).unwrap();
                font
        });
       
        self.engine.register_fn("new_image", move |path_string: &str| {
            path_string.to_owned()
        });
        
        let graphics_calls = self.graphics_calls.clone();
        self.engine.register_fn("draw_image", move |path_string: &str, x: f32, y: f32| {
            graphics_calls.borrow_mut().push(GraphicsCalls::DrawImage((x, y).into(), path_string.to_owned()));
        });

        let graphics_calls = self.graphics_calls.clone();
        self.engine.register_fn("draw_image", move |path_string: &str, x: f32, y: f32, w: f32, h: f32| {
            graphics_calls.borrow_mut().push(GraphicsCalls::DrawRectangleImageTinted(
                Rectangle::new((x, y).into(), (x + w, y + h).into()),
                path_string.to_owned(),
                Color::WHITE,
            ));
        });
        
        let graphics_calls = self.graphics_calls.clone();
        self.engine.register_fn("draw_image", move |path_string: &str, x: f32, y: f32, w: f32, h: f32, alpha: f32| {
            graphics_calls.borrow_mut().push(GraphicsCalls::DrawRectangleImageTinted(
                Rectangle::new((x, y).into(), (x + w, y + h).into()),
                path_string.to_owned(),
                Color::from_rgba(1.0, 1.0, 1.0, alpha),
            ));
        });        
        
        let result = self.engine.eval_ast_with_scope::<()>(&mut self.scope, &self.ast);
        if let Err(err) = result {
            dbg!(&err);
        }               
    }
    
    pub fn get_resolution(&self) -> Option<(u32, u32)> {       
        match self.scope.get_value::<Array>("resolution") {
            Some(a) => {
                match (a[0].clone().try_cast::<i64>(), a[1].clone().try_cast::<i64>()) {
                    (Some(x), Some(y)) => Some((x as u32, y as u32)),
                    _ => None,
                }
            },
            None => None
        }
    }
    
    fn get_image_handle(&mut self, path_string: &str, graphics: &mut Graphics2D) -> ImageHandle {
        let mut created = false;
        let image_handle = match self.image_handles.borrow_mut().get_mut(path_string) {
            Some(image_handle) => image_handle.clone(),
            None => {
                let mut path = self.root_path.borrow().clone();
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