use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use boa_engine::{JsNativeError, JsResult, JsValue, JsError};
use boa_engine::{Context, Module, Source};
use boa_engine::builtins::promise::PromiseState;
use boa_engine::module::{ModuleLoader, SimpleModuleLoader};
use boa_engine::object::builtins::{JsArray, JsFunction};
use boa_engine::property::{Attribute, PropertyKey};
use boa_engine::value::TryFromJs;
use boa_runtime::Console;

use crate::js_draw::{register_fns_and_types, GraphicsCalls};

pub struct JsEnv {
    context: Context<'static>,
    module: Module,
    graphics_calls: Rc<RefCell<Vec<GraphicsCalls>>>,
}

impl JsEnv {
    pub fn new(
        app_path: &Path,
        watches: &Rc<RefCell<HashMap<PathBuf, JsFunction>>>) -> JsResult<Self>
{
        let graphics_calls = Rc::new(RefCell::new(vec![]));
        let (context, module) = JsEnv::create_context(app_path, &graphics_calls, watches)?;
        
        Ok(JsEnv {
            context,
            module,
            graphics_calls,
        })
    }
    
    pub fn graphics_calls(&self) -> &Rc<RefCell<Vec<GraphicsCalls>>> {
        &self.graphics_calls
    }
    
    pub fn clear_graphics_calls(&self) {
        self.graphics_calls.borrow_mut().clear();
    }
    
    pub fn context_mut(&mut self) -> &mut Context<'static> {
        &mut self.context
    }
    
    pub fn create_context(
        app_path: &Path,
        graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>,
        watches: &Rc<RefCell<HashMap<PathBuf, JsFunction>>>
        ) -> JsResult<(Context<'static>, Module)>
    {
        let loader = Rc::new(SimpleModuleLoader::new(Path::new(&app_path))?);
        let dyn_loader: Rc<dyn ModuleLoader> = loader.clone();
        let mut context = Context::builder().module_loader(dyn_loader).build()?;
        
        let app_path_str = app_path.to_str().unwrap();      
        context.global_object().set("app_path", app_path_str, true, &mut context).unwrap();        
        
        register_fns_and_types(&mut context, graphics_calls, watches);        
        
        let console = Console::init(&mut context);
        context.register_global_property(Console::NAME, console, Attribute::all())?;
        
        let mut main = app_path.to_path_buf();
        main.push("main.js");
        let source = Source::from_filepath(&main).unwrap();
        let module = Module::parse(source, None, &mut context)?;
        loader.insert(Path::new("main.js").to_owned(), module.clone());
        let promise = module.load_link_evaluate(&mut context)?;
        context.run_jobs();
        
        if let PromiseState::Rejected(err) = promise.state().unwrap() {
            println!("Promise error: {}", err.display());
        } else {
            println!("Success");
        }
        
        Ok((context, module))
    }
    
    pub fn call_init(&mut self) -> Result<(), JsError> {       
        let namespace = self.module.namespace(&mut self.context);
        let init = namespace
            .get("init", &mut self.context)?
            .as_callable()
            .cloned()
            .ok_or_else(|| JsNativeError::typ().with_message("main.js must export init function!"))?;
        
        init.call(&boa_engine::JsValue::Null, &[], &mut self.context)?;
        Ok(())
    }

    pub fn call_draw(&mut self, dt: f32) -> Result<(), JsError> {
        let namespace = self.module.namespace(&mut self.context);
        let draw = namespace
            .get("draw", &mut self.context)?
            .as_callable()
            .cloned()
            .ok_or_else(|| JsNativeError::typ().with_message("main.js must export draw function!"))?;

        draw.call(
            &boa_engine::JsValue::Null,
            &[JsValue::Rational(dt as f64)],
            &mut self.context
        )?;
        Ok(())
    }
    
    pub fn get_value<T, K>(&mut self, key: K) -> Result<T, JsError>
        where K: Into<PropertyKey>,
              T: TryFromJs
    {       
        let value = self.module.namespace(&mut self.context)
            .get(key, &mut self.context)?
            .try_js_into::<T>(&mut self.context)?;
        
        Ok(value)
    }

    pub fn get_array<T, K>(&mut self, key: K) -> Result<Vec<T>, JsError>
        where K: Into<PropertyKey>,
              T: TryFromJs
    {       
        let value = self.module.namespace(&mut self.context)
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
