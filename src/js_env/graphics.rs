use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::{Context, JsNativeError, JsResult, NativeFunction, JsError, JsValue};
use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::builtins::{JsFunction, JsArray};
use boa_engine::property::Attribute;
use boa_engine::value::TryFromJs;
use boa_gc::{Trace, Finalize};
use boa_runtime::Console;
use speedy2d::color::Color;
use speedy2d::dimen::{Vec2, UVec2};
use speedy2d::shape::Rectangle;
use speedy2d::font::{Font, TextOptions, TextLayout, FormattedTextBlock};

pub enum GraphicsCalls {
    ClearScreenBlack,
    ClearScreen(Color),
    DrawRectangle(Rectangle, Color),
    DrawText(Vec2, Color, Rc<FormattedTextBlock>),
    DrawImage(Vec2, String),
    DrawRectangleImageTinted(Rectangle, String, Color),
    PushOffset(Vec2),
    PopOffset(),
    SetResolution(UVec2),
}

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

#[derive(Trace, Finalize, Clone)]
struct JsFont {
    #[unsafe_ignore_trace]
    font: Font,
    #[unsafe_ignore_trace]
    cache: HashMap<BlockCacheKey, Rc<FormattedTextBlock>>,
    test: i32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct BlockCacheKey {
    text: String,
    scale: i32,
}

impl BlockCacheKey {
    fn new(text: &str, scale: f32) -> Self {
        BlockCacheKey {
            text: text.to_owned(),
            scale: (scale * 100.) as i32,
        }
    }
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
        let cache = HashMap::new();        
        
        Ok(JsFont{font, cache, test: 10})
    }
    
    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        class.method("cacheLength", 0, NativeFunction::from_fn_ptr(Self::cache_length));
        Ok(())
    }
}

impl JsFont {   
    // todo Cold cache items should be pruned eventually
    fn layout_text(&mut self, text: &str, scale: f32) -> Rc<FormattedTextBlock> {
        let key = BlockCacheKey::new(text, scale);
                               
        match self.cache.get(&key) {
            Some(block) => block.clone(),
            None => {
                let block = self.font.layout_text(text, scale, TextOptions::new());
                self.cache.insert(key, block.clone());
                block
            }
        }
    }
    
    fn cache_length(this: &JsValue, _: &[JsValue], _: &mut Context<'_>) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(js_font) = object.downcast_ref::<JsFont>() {
                return Ok(JsValue::Integer(js_font.cache.len() as i32));
            }
        }
        Err(JsNativeError::typ().with_message("'this' is not a JsFont object").into())
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

pub fn register_fns_and_types(
    context: &mut Context,
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>
) {
    let console = Console::init(context);
    context.register_global_property(Console::NAME, console, Attribute::all())
        .expect("Unable to create console object");
    
    context.register_global_class::<JsColor>().expect("Could not register JsColor");
    context.register_global_class::<JsFont>().expect("Could not register JsFont");
    context.register_global_class::<JsImage>().expect("Could not register Image");
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        context.register_global_callable(
            "clear_screen", 1, NativeFunction::from_closure(move |this, args, context| {
                clear_screen(&graphics_calls_, this, args, context)
            })
        ).unwrap();
    }
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        context.register_global_callable(
            "draw_rectangle", 1, NativeFunction::from_closure(move |this, args, context| {
                draw_rectangle(&graphics_calls_, this, args, context)
            })
        ).unwrap();
    }
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        context.register_global_callable(
            "draw_text", 1, NativeFunction::from_closure(move |this, args, context| {
                draw_text(&graphics_calls_, this, args, context)
            })
        ).unwrap();
    }

    context.register_global_callable(
        "size_text", 1, NativeFunction::from_copy_closure(move |this, args, context| {
            size_text(this, args, context)
        })
    ).unwrap();
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        context.register_global_callable(
            "draw_image", 1, NativeFunction::from_closure(move |this, args, context| {
                draw_image(&graphics_calls_, this, args, context)
            })
        ).unwrap();
    }

    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        context.register_global_callable(
            "with_offset", 1, NativeFunction::from_closure(move |this, args, context| {
                with_offset(&graphics_calls_, this, args, context)
            })
        ).unwrap();
    }
    
    let graphics_calls_ = graphics_calls.clone();
    unsafe {
        context.register_global_callable(
            "set_resolution", 1, NativeFunction::from_closure(move |this, args, context| {
                set_resolution(&graphics_calls_, this, args, context)
            })
        ).unwrap();
    }
}

fn clear_screen(
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    _this: &JsValue, args: &[JsValue], _context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.is_empty() {
        graphics_calls.borrow_mut().push(GraphicsCalls::ClearScreenBlack);
    } else {
        let c = args[0].as_object()
            .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
            .downcast_ref::<JsColor>()
            .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
            .clone();

        graphics_calls.borrow_mut().push(GraphicsCalls::ClearScreen(c.into()));        
    }
    
    Ok(JsValue::Undefined)    
}

fn draw_rectangle(
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    _this: &JsValue, args: &[JsValue], context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.len() < 5 {
        return Err(JsNativeError::typ().with_message("Too few arguments for draw_rectangle").into());
    }

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
    graphics_calls.borrow_mut().push(GraphicsCalls::DrawRectangle(r, c.into()));
    Ok(JsValue::Undefined)
}

fn draw_text(
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    _this: &JsValue, args: &[JsValue], context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.len() < 6 {
        return Err(JsNativeError::typ().with_message("Too few arguments for draw_text").into());
    }

    let mut js_font = args[0].as_object()
        .ok_or(JsNativeError::typ().with_message("Expected a Font"))?
        .downcast_mut::<JsFont>()
        .ok_or(JsNativeError::typ().with_message("Expected a Font"))?;
        
    let text = args[1].try_js_into::<String>(context)?;   
    let x = args[2].try_js_into::<f64>(context)? as f32;
    let y = args[3].try_js_into::<f64>(context)? as f32;
    let s = args[4].try_js_into::<f64>(context)? as f32;

    let c = args[5].as_object()
        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
        .downcast_ref::<JsColor>()
        .ok_or(JsNativeError::typ().with_message("Expected a Color"))?
        .clone();
                                            
    let block = js_font.layout_text(&text, s);
    graphics_calls.borrow_mut().push(
        GraphicsCalls::DrawText((x, y).into(), c.into(), block)
    );

    Ok(JsValue::Undefined)
}

fn size_text(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue>
{
    if args.len() < 3 {
        return Err(JsNativeError::typ().with_message("Too few arguments for size_text").into());
    }
    
    let mut js_font = args[0].as_object()
        .ok_or(JsNativeError::typ().with_message("Expected a Font"))?
        .downcast_mut::<JsFont>()
        .ok_or(JsNativeError::typ().with_message("Expected a Font"))?;

    let text = args[1].try_js_into::<String>(context)?;
    let s = args[2].try_js_into::<f64>(context)? as f32;
    
    let block = js_font.layout_text(&text, s);
    let size = block.size();
    
    let array = JsArray::new(context);
    array.push(size.x, context)?;
    array.push(size.y, context)?;
    
    Ok(JsValue::Object(array.into()))
}

fn draw_image(
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    _this: &JsValue, args: &[JsValue], context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.len() == 3 {
        let js_image = args[0].as_object()
            .ok_or(JsNativeError::typ().with_message("Expected an Image"))?
            .downcast_ref::<JsImage>()
            .ok_or(JsNativeError::typ().with_message("Expected a Image"))?
            .clone();
        
        let x = args[1].try_js_into::<f64>(context)? as f32;
        let y = args[2].try_js_into::<f64>(context)? as f32;
                                                
        graphics_calls.borrow_mut().push(
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
                                                
        graphics_calls.borrow_mut().push(
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

        graphics_calls.borrow_mut().push(
            GraphicsCalls::DrawRectangleImageTinted(
                Rectangle::new((x, y).into(), (x + w, y + h).into()),
                js_image.path.to_str().unwrap().to_owned(),
                Color::from_rgba(1., 1., 1., a),
            )
        );
    } else {
        return Err(JsNativeError::typ().with_message("Unexpected number of arguments for draw_image").into());
    }
    Ok(JsValue::Undefined)
}

fn with_offset(
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    this: &JsValue, args: &[JsValue], context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.len() < 3 {
        return Err(JsNativeError::typ().with_message("Too few arguments for with_offset").into());
    }
    
    let x = args[0].try_js_into::<f64>(context)? as f32;
    let y = args[1].try_js_into::<f64>(context)? as f32;                
    let func = args[2].try_js_into::<JsFunction>(context)?;
    
    graphics_calls.borrow_mut().push(
        GraphicsCalls::PushOffset((x, y).into())
    );

    let call_result = func.call(this, args, context);
    graphics_calls.borrow_mut().push(GraphicsCalls::PopOffset());
    call_result
}

fn set_resolution(
    graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
    _this: &JsValue, args: &[JsValue], context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.len() < 2 {
        return Err(JsNativeError::typ().with_message("Too few arguments for set_resolution").into());
    }
    
    let x = args[0].try_js_into::<f64>(context)? as u32;
    let y = args[1].try_js_into::<f64>(context)? as u32;
    
    graphics_calls.borrow_mut().push(
        GraphicsCalls::SetResolution((x, y).into())
    );

    Ok(JsValue::Undefined)
}
