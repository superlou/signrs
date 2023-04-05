use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;

use speedy2d::Window;
use speedy2d::shape::Rectangle;
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use rhai::{Engine, Scope, AST};

struct SignWindowHandler {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
    last_frame_time: Instant,
    graphics_calls: Rc<RefCell<Vec<GraphicsCalls>>>,
}

enum GraphicsCalls {
    ClearScreen(Color),
    DrawRectangle(Rectangle, Color),
}

impl WindowHandler for SignWindowHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        let dt = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();
      
        let result = self.engine.call_fn::<()>(&mut self.scope, &mut self.ast, "draw", (dt,));
        if let Err(err) = result {
            dbg!(&err);
        }

        for call in self.graphics_calls.borrow().iter() {
            match call {
                GraphicsCalls::ClearScreen(c) => graphics.clear_screen(*c),
                GraphicsCalls::DrawRectangle(r, c) => graphics.draw_rectangle(r.clone(), *c),
            }
        }
        self.graphics_calls.borrow_mut().clear();
        
        helper.request_redraw();
    }
    
    fn on_start(&mut self, _helper: &mut WindowHelper<()>, _info: speedy2d::window::WindowStartupInfo) {
        self.setup_engine();
    }
}

impl SignWindowHandler {
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
    
        self.engine.register_type_with_name::<Color>("Color")
            .register_fn("new_color_from_rgb", Color::from_rgb);

        self.engine.register_type_with_name::<Color>("Color")
            .register_fn("new_color_from_rgba", Color::from_rgba);        
                        
        let result = self.engine.eval_ast_with_scope::<()>(&mut self.scope, &self.ast);
        if let Err(err) = result {
            dbg!(&err);
        }               
    }
}

fn main() {
    println!("Starting...");
    let engine = Engine::new();   
    let ast = engine.compile_file("examples/display1/main.rhai".into()).unwrap();
    let scope = Scope::new();
    
    let window = Window::new_centered("Title", (640, 480)).unwrap();
        
    let handler = SignWindowHandler {
        engine, ast, scope,
        last_frame_time: Instant::now(),
        graphics_calls: Rc::new(RefCell::new(vec![])),
    };

    window.run_loop(handler);
}
