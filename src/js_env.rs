use std::fs::read_to_string;
use std::path::PathBuf;
use std::path::Path;

use boa_engine::{Context, JsValue, Source, JsError};
use thiserror::Error;

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
    
    pub fn load_json(path: impl AsRef<Path>, context: &mut Context) -> Result<JsValue, JsError> {
        let json_text = read_to_string(path).unwrap_or("{}".to_owned());
        let Ok(json_data) = serde_json::from_str(&json_text) else {return Ok(JsValue::Undefined)};
        JsValue::from_json(&json_data, context)
    }
}
