use std::fs::read_to_string;
use std::path::PathBuf;
use std::path::Path;

use boa_engine::JsNativeError;
use boa_engine::JsResult;
use boa_engine::NativeFunction;
use boa_engine::object::builtins::JsArray;
use boa_engine::property::PropertyKey;
use boa_engine::value::TryFromJs;
use boa_engine::{Context, JsValue, Source, JsError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsEnvError {
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
        
        context.register_global_callable(
            "include", 1, NativeFunction::from_copy_closure(move |this, args, context| {
                include_js(this, args, context)
            })
        ).unwrap();
        
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
    
    pub fn get_global<T, K>(&mut self, key: K) -> Result<T, JsError>
        where K: Into<PropertyKey>,
              T: TryFromJs
    {       
        let value = self.context.global_object()
            .get(key, &mut self.context)?
            .try_js_into::<T>(&mut self.context)?;
        
        Ok(value)
    }

    pub fn get_global_array<T, K>(&mut self, key: K) -> Result<Vec<T>, JsError>
        where K: Into<PropertyKey>,
              T: TryFromJs
    {       
        let value = self.context.global_object()
            .get(key, &mut self.context)?
            .try_js_into::<JsArray>(&mut self.context)?;
        
        let length = value.length(&mut self.context)? as i64;
        let mut vec: Vec<T> = vec![];
        
        for i in 0..length {
            vec.push(
                value.at(i, &mut self.context)?
                    .try_js_into::<T>(&mut self.context)?
                )
        }

        Ok(vec)
    }
            
    pub fn load_json(path: impl AsRef<Path>, context: &mut Context) -> Result<JsValue, JsError> {
        let json_text = read_to_string(path).unwrap_or("{}".to_owned());
        let Ok(json_data) = serde_json::from_str(&json_text) else {return Ok(JsValue::Undefined)};
        JsValue::from_json(&json_data, context)
    }
}

fn include_js(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if args.len() < 1 {
        return Err(JsNativeError::typ().with_message("Too few arguments for include").into())
    }
    
    let file_path = args[0].try_js_into::<String>(context)?;
    
    let app_path = context.global_object()
        .get("app_path", context)?
        .try_js_into::<String>(context)?;
    let mut full_path = PathBuf::from(app_path);
    full_path.push(&file_path);
    
    let src = Source::from_filepath(&full_path).map_err(|_| {
        JsNativeError::typ().with_message(format!("Unable to include source: {}", &file_path))}
    )?;
    
    context.eval_script(src)?;
    
    Ok(JsValue::Undefined)
}