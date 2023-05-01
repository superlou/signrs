use std::{path::Path};
use std::rc::Rc;
use std::cell::RefCell;

use boa_engine::{Context, JsValue, JsResult, Source, NativeFunction, class::{Class, ClassBuilder}, property::Attribute, value::TryFromJs, JsError};
use boa_gc::{GcRefCell, Trace, Finalize, empty_trace};
use crate::window_handler::GraphicsCalls;

pub struct JsEnv {
    context: Context<'static>,
}

impl JsEnv {
    pub fn new(app_path: &Path) -> Self {
        let mut context = Context::default();
        
        let mut main = app_path.clone().to_owned();
        main.push("main.js");
        
        let source = Source::from_filepath(&main).unwrap();
        context.eval_script(source).unwrap();
        
        JsEnv {
            context,
        }
    }
    
    pub fn call_init(&mut self) {
        let global_object = self.context.global_object().clone();
        let init = global_object.get("init", &mut self.context).unwrap();
        let init = init.as_object().unwrap();
        init.call(&boa_engine::JsValue::Null, &[], &mut self.context).unwrap();
    }
    
    pub fn call_draw(&mut self) {
        let global_object = self.context.global_object().clone();
        let init = global_object.get("draw", &mut self.context).unwrap();
        let init = init.as_object().unwrap();
        init.call(&boa_engine::JsValue::Null, &[], &mut self.context).unwrap();        
    }
}

use speedy2d::color::Color;

#[derive(Debug, Trace, Finalize, TryFromJs, Clone)]
struct JsColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl Class for JsColor {
    const NAME: &'static str = "Color";
    const LENGTH: usize = 4;
    
    fn constructor(_this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<Self> {
        let color = match args.len() {
            3 => JsColor {
                r: args[0].as_number().ok_or(JsError::from_opaque("r must be a number".into()))?,
                g: args[1].as_number().ok_or(JsError::from_opaque("g must be a number".into()))?,
                b: args[2].as_number().ok_or(JsError::from_opaque("b must be a number".into()))?,
                a: 1.0
            },
            4 => JsColor {
                r: args[0].as_number().ok_or(JsError::from_opaque("r must be a number".into()))?,
                g: args[1].as_number().ok_or(JsError::from_opaque("g must be a number".into()))?,
                b: args[2].as_number().ok_or(JsError::from_opaque("b must be a number".into()))?,
                a: args[3].as_number().ok_or(JsError::from_opaque("a must be a number".into()))?,
            },
            _ => JsColor {r: 0., g: 0., b: 0., a: 0.},
        };

        Ok(color)
    }

    fn init(_class: &mut ClassBuilder) -> JsResult<()> {
        Ok(())
    }
}

impl From<JsColor> for Color {
    fn from(color: JsColor) -> Color {
        Color::from_rgba(color.r as f32, color.g as f32, color.b as f32, color.a as f32)
    }
}

pub fn register_fns_and_types(
    script_env: &mut JsEnv,
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>
) {
    script_env.context.register_global_class::<JsColor>().expect("Could not register JsColor");
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        script_env.context.register_global_callable(
            "clear_screen",
            1,
            NativeFunction::from_closure(move |_this, args, _context| {
                if args.len() > 0 {
                    let c = args[0].as_object().unwrap().downcast_ref::<JsColor>().unwrap().clone();
                    graphics_calls_.borrow_mut().push(GraphicsCalls::ClearScreen(c.into()));
                } else {
                    graphics_calls_.borrow_mut().push(GraphicsCalls::ClearScreenBlack);
                }
                
                Ok(JsValue::Undefined)
        })).unwrap();
    }
}
