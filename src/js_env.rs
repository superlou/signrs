use std::path::PathBuf;
use std::str::FromStr;
use std::{path::Path};
use std::rc::Rc;
use std::cell::RefCell;

use boa_engine::JsNativeError;
use boa_engine::{Context, JsValue, JsResult, Source, NativeFunction, class::{Class, ClassBuilder}, property::Attribute, value::TryFromJs, JsError};
use boa_gc::{GcRefCell, Trace, Finalize, empty_trace};
use boa_runtime::Console;
use thiserror::Error;

use crate::window_handler::GraphicsCalls;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("EvalAltError: {0}")]
    EvalAltError(#[from] JsError),
}

pub struct JsEnv {
    context: Context<'static>,
    app_path: PathBuf,
}

impl JsEnv {
    pub fn new(app_path: &Path) -> Self {
        let mut context = Context::default();
        
        let app_path_str = app_path.to_str().unwrap();      
        context.global_object().set("app_path", app_path_str, true, &mut context).unwrap();
        
        JsEnv {
            context,
            app_path: app_path.to_owned(),
        }
    }
    
    pub fn call_init(&mut self) -> Result<(), JsError> {
        let mut main = self.app_path.clone().to_owned();
        main.push("main.js");
        let source = Source::from_filepath(&main).unwrap();
        self.context.eval_script(source)?;           
        
        let global_object = self.context.global_object().clone();
        let init = global_object.get("init", &mut self.context).unwrap();
        let init = init.as_object().unwrap();
        init.call(&boa_engine::JsValue::Null, &[], &mut self.context)?;
        Ok(())
    }

    pub fn call_draw(&mut self, dt: f32) -> Result<(), JsError> {
        let global_object = self.context.global_object().clone();
        let init = global_object.get("draw", &mut self.context).unwrap();
        let init = init.as_object().unwrap();
        init.call(
            &boa_engine::JsValue::Null,
            &[JsValue::Rational(dt as f64)],
            &mut self.context
        )?;
        Ok(())
    }
}

use speedy2d::color::Color;
use speedy2d::shape::Rectangle;
use speedy2d::font::{Font, TextOptions, TextLayout};

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

#[derive(Debug, Trace, Finalize, Clone)]
struct JsFont {
    #[unsafe_ignore_trace]
    font: Font,
}

impl Class for JsFont {
    const NAME: &'static str = "Font";
    const LENGTH: usize = 1;

    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
        let font_path = args[0].try_js_into::<String>(context)?;
        let mut full_path = PathBuf::from_str(
            &context.global_object().get("app_path", context).unwrap().try_js_into::<String>(context).unwrap()
        ).unwrap();
        
        full_path.push(font_path);
        
        let bytes = std::fs::read(full_path).unwrap();
        let font = Font::new(&bytes).unwrap();
        
        Ok(JsFont{font})
    }
    
    fn init(_class: &mut ClassBuilder) -> JsResult<()> {
        Ok(())
    }
}

pub fn register_fns_and_types(
    script_env: &mut JsEnv,
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>
) {
    let console = Console::init(&mut script_env.context);
    script_env.context.register_global_property(Console::NAME, console, Attribute::all())
        .expect("Unable to create console object");
    
    script_env.context.register_global_class::<JsColor>().expect("Could not register JsColor");
    script_env.context.register_global_class::<JsFont>().expect("Could not register JsFont");
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        script_env.context.register_global_callable(
            "clear_screen",
            1,
            NativeFunction::from_closure(move |_this, args, _context| {
                if args.len() >= 1 {
                    let c = args[0].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
                        .downcast_ref::<JsColor>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
                        .clone();

                    graphics_calls_.borrow_mut().push(GraphicsCalls::ClearScreen(c.into()));
                } else {
                    graphics_calls_.borrow_mut().push(GraphicsCalls::ClearScreenBlack);
                }
                
                Ok(JsValue::Undefined)
            })
        ).unwrap();
    }
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        script_env.context.register_global_callable(
            "draw_rectangle",
            1,
            NativeFunction::from_closure(move |_this, args, context| {
                if args.len() >= 5 {
                    let x = args[0].try_js_into::<f64>(context)? as f32;
                    let y = args[1].try_js_into::<f64>(context)? as f32;
                    let w = args[2].try_js_into::<f64>(context)? as f32;
                    let h = args[3].try_js_into::<f64>(context)? as f32;

                    let c = args[4].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
                        .downcast_ref::<JsColor>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
                        .clone();

                    let r = Rectangle::from_tuples((x, y), (x + w, y + h));
                    graphics_calls_.borrow_mut().push(GraphicsCalls::DrawRectangle(r, c.into()));
                }
                Ok(JsValue::Undefined)
            })
        ).unwrap();
    }
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        script_env.context.register_global_callable(
            "draw_text",
            1,
            NativeFunction::from_closure(move |_this, args, context| {
                if args.len() >= 5 {
                    let js_font = args[0].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected a Font"))?
                        .downcast_ref::<JsFont>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Font"))?
                        .clone();
                        
                    let text = args[1].try_js_into::<String>(context)?;
                    
                    let x = args[2].try_js_into::<f64>(context)? as f32;
                    let y = args[3].try_js_into::<f64>(context)? as f32;                    

                    let c = args[4].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
                        .downcast_ref::<JsColor>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
                        .clone();                    
                                                            
                    let block = js_font.font.layout_text(&text, 18., TextOptions::new());
                    graphics_calls_.borrow_mut().push(
                        GraphicsCalls::DrawText((x, y).into(), c.into(), block)
                    );
                }
                Ok(JsValue::Undefined)
            })
        ).unwrap();
    }    
}
