use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::sync::mpsc;
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
use local_ip_address::local_ip;
use notify::{Watcher, RecursiveMode};
use tracing::warn;

mod graphics;
mod files;
pub use graphics::GraphicsCalls;

pub struct JsEnv {
    app_path: PathBuf,
    context: Context<'static>,
    module: Module,
    graphics_calls: Rc<RefCell<Vec<GraphicsCalls>>>,
    
    #[allow(deprecated)]
    watches: Rc<RefCell<HashMap<PathBuf, JsFunction>>>,
    #[allow(dead_code)] // Required to keep watcher in scope
    watcher: Box<dyn Watcher>,
    file_change_rx: mpsc::Receiver<PathBuf>,
    _file_change_tx: mpsc::Sender<PathBuf>,
}

const FALLBACK_SCRIPT: &str = r###"
    export function init() {}

    export function draw(dt) {
        clear_screen(new Color(0, 0, 0));
    }
"###;

impl JsEnv {
    pub fn new(app_path: &Path) -> Self
    {
        let (tx, rx) = mpsc::channel();
        let tx_for_watcher = tx.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {           
            match res {
                Ok(event) if event.kind.is_modify() => {
                    for path_buf in event.paths {
                        let cwd = std::env::current_dir().unwrap();
                        let path = path_buf.strip_prefix(&cwd).unwrap();
                        let _ = tx_for_watcher.send(path.to_owned());
                    }
                },
                Ok(_event) => {} // Ignore other events
                Err(err) => warn!("Watch error: {:?}", err),
            }
        }).unwrap();
         
        watcher.watch(app_path.as_ref(), RecursiveMode::Recursive).unwrap();
             
        let watches = Rc::new(RefCell::new(HashMap::new()));
        
        let graphics_calls = Rc::new(RefCell::new(vec![]));
        let (context, module) = JsEnv::create_context(app_path, &graphics_calls, &watches)
            .unwrap_or_else(|err| {
                dbg!(err);
                JsEnv::create_fallback_context(&graphics_calls)
            }
        );
        
        JsEnv {
            app_path: app_path.to_owned(),
            context,
            module,
            graphics_calls,
            watches,
            watcher: Box::new(watcher),
            _file_change_tx: tx,
            file_change_rx: rx,
        }
    }
    
    /// Create a simple context and module that should never fail
    pub fn create_fallback_context(graphics_calls: &Rc<RefCell<Vec<GraphicsCalls>>>)
        -> (Context<'static>, Module)
    {
        let mut context = Context::default();
        graphics::register_fns_and_types(&mut context, graphics_calls);
        let source = Source::from_bytes(FALLBACK_SCRIPT);
        let module = Module::parse(source, None, &mut context).unwrap();
        let promise = module.load_link_evaluate(&mut context).unwrap();
        context.run_jobs();

        if let PromiseState::Rejected(err) = promise.state().unwrap() {
            println!("Promise error: {}", err.display());
            panic!();
        }
        
        (context, module)
    }
    
    pub fn graphics_calls(&self) -> &Rc<RefCell<Vec<GraphicsCalls>>> {
        &self.graphics_calls
    }
    
    pub fn clear_graphics_calls(&self) {
        self.graphics_calls.borrow_mut().clear();
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
        context.global_object().set("app_path", app_path_str, true, &mut context)?;
        
        let hostname = match hostname::get() {
            Ok(str) => str.to_str().unwrap_or("(unknown)").to_owned(),
            Err(_) => "(unknown)".to_owned(),
        };
        context.global_object().set("hostname", hostname, true, &mut context)?;
        
        let local_ip = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(_) => "(unknown)".to_owned(),
        };
        context.global_object().set("localIp", local_ip, true, &mut context)?;
        
        graphics::register_fns_and_types(&mut context, graphics_calls);
        files::register_fns_and_types(&mut context, watches);
        
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
            return Err(JsNativeError::eval().with_message(err.display().to_string()).into());
        }
        
        Ok((context, module))
    }
    
    pub fn call_module_init(module: &Module, context: &mut Context) -> Result<(), JsError> {
        let namespace = module.namespace(context);
        let init = namespace
            .get("init", context)?
            .as_callable()
            .cloned()
            .ok_or_else(|| JsNativeError::typ().with_message("main.js must export init function!"))?;
        
        init.call(&boa_engine::JsValue::Null, &[], context)?;
        Ok(())        
    }
    
    pub fn call_init(&mut self) -> Result<(), JsError> {       
        JsEnv::call_module_init(&self.module, &mut self.context)
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
    
    pub fn _get_value<T, K>(&mut self, key: K) -> Result<T, JsError>
        where K: Into<PropertyKey>,
              T: TryFromJs
    {       
        let value = self.module.namespace(&mut self.context)
            .get(key, &mut self.context)?
            .try_js_into::<T>(&mut self.context)?;
        
        Ok(value)
    }

    pub fn _get_array<T, K>(&mut self, key: K) -> Result<Vec<T>, JsError>
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
