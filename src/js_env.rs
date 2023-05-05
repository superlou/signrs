use std::fs::{read_to_string, self};
use std::path::PathBuf;
use std::str::FromStr;
use std::{path::Path};
use std::rc::Rc;
use std::cell::RefCell;

use boa_engine::JsNativeError;
use boa_engine::object::builtins::JsFunction;
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
    pub context: Context<'static>,
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

#[derive(Debug, Trace, Finalize, Clone)]
struct JsImage {
    path: PathBuf,
}

impl Class for JsImage {
    const NAME: &'static str = "Image";
    const LENGTH: usize = 1;

    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context<'_>) -> JsResult<Self> {
        let path = args[0].try_js_into::<String>(context)?;
        let path = PathBuf::from_str(&path).unwrap();
        Ok(Self{path})
    }
    
    fn init(_class: &mut ClassBuilder<'_, '_>) -> JsResult<()> {
        Ok(())
    }
}

use std::collections::HashMap;

pub fn register_fns_and_types(
    script_env: &mut JsEnv,
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    watches: &Rc<RefCell<HashMap<PathBuf, JsFunction>>>
) {
    let console = Console::init(&mut script_env.context);
    script_env.context.register_global_property(Console::NAME, console, Attribute::all())
        .expect("Unable to create console object");
    
    script_env.context.register_global_class::<JsColor>().expect("Could not register JsColor");
    script_env.context.register_global_class::<JsFont>().expect("Could not register JsFont");
    script_env.context.register_global_class::<JsImage>().expect("Could not register Image");
    
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
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        script_env.context.register_global_callable(
            "draw_image",
            1,
            NativeFunction::from_closure(move |_this, args, context| {
                if args.len() == 3 {
                    let js_image = args[0].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected an Image"))?
                        .downcast_ref::<JsImage>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Image"))?
                        .clone();
                    
                    let x = args[1].try_js_into::<f64>(context)? as f32;
                    let y = args[2].try_js_into::<f64>(context)? as f32;
                                                            
                    graphics_calls_.borrow_mut().push(
                        GraphicsCalls::DrawImage((x, y).into(), js_image.path.to_str().unwrap().to_owned())
                    );
                } else if args.len() == 5 {
                    let js_image = args[0].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected an Image"))?
                        .downcast_ref::<JsImage>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Image"))?
                        .clone();
                    
                    let x = args[1].try_js_into::<f64>(context)? as f32;
                    let y = args[2].try_js_into::<f64>(context)? as f32;
                    let w = args[3].try_js_into::<f64>(context)? as f32;
                    let h = args[4].try_js_into::<f64>(context)? as f32;                                                            
                                                            
                    graphics_calls_.borrow_mut().push(
                        GraphicsCalls::DrawRectangleImageTinted(
                            Rectangle::new((x, y).into(), (x + w, y + h).into()),
                            js_image.path.to_str().unwrap().to_owned(),
                            Color::WHITE,
                        )
                    );
                } else if args.len() == 6 {
                    let js_image = args[0].as_object()
                        .ok_or(JsNativeError::typ().with_message("Expected an Image"))?
                        .downcast_ref::<JsImage>()
                        .ok_or(JsNativeError::typ().with_message("Expected a Image"))?
                        .clone();
                    
                    let x = args[1].try_js_into::<f64>(context)? as f32;
                    let y = args[2].try_js_into::<f64>(context)? as f32;
                    let w = args[3].try_js_into::<f64>(context)? as f32;
                    let h = args[4].try_js_into::<f64>(context)? as f32;
                    let a = args[5].try_js_into::<f64>(context)? as f32;                                                            
                                                            
                    graphics_calls_.borrow_mut().push(
                        GraphicsCalls::DrawRectangleImageTinted(
                            Rectangle::new((x, y).into(), (x + w, y + h).into()),
                            js_image.path.to_str().unwrap().to_owned(),
                            Color::from_rgba(1., 1., 1., a),
                        )
                    );
                }                     
                Ok(JsValue::Undefined)
            })
        ).unwrap();
    }
    
    let _watches = watches.clone();
    unsafe {
        script_env.context.register_global_callable(
            "watch_json",
            2,
            NativeFunction::from_closure(move |_this, args, context| {
                if args.len() < 2 {
                    return Err(JsNativeError::typ().with_message("Not enough arguments").into());
                }
                
                let app_path = context.global_object().get("app_path", context)?
                    .try_js_into::<String>(context)?;
                let mut full_path = PathBuf::from(app_path);
                
                let path = args[0].try_js_into::<String>(context)?;
                full_path.push(path);
                let canonical_path = fs::canonicalize(&full_path).unwrap();
                
                let callback = args[1].try_js_into::<JsFunction>(context)?;

                // todo Keeping the callback outside the JsEnv seems to cause core dump on quit
                _watches.borrow_mut().insert(canonical_path, callback);

                let json_text = read_to_string(&full_path).unwrap_or("{}".to_owned());
                let json_data: serde_json::Value = serde_json::from_str(&json_text).unwrap();
                Ok(JsValue::from_json(&json_data, context)?)
            })
        ).unwrap();
    }
}
