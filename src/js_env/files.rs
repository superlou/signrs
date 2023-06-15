use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::{Context, JsNativeError, JsResult, JsValue, NativeFunction};
use boa_engine::object::builtins::JsFunction;

use crate::js_env::JsEnv;
use crate::iter_util::iter_unique;

pub fn register_fns_and_types(
    context: &mut Context,
    watches: &Rc<RefCell<HashMap<PathBuf, JsFunction>>>
) {
    let watches_ = watches.clone();
    unsafe {
        context.register_global_callable(
            "watch_json", 2, NativeFunction::from_closure(move |this, args, context| {
                watch_json(&watches_, this, args, context)
            })
        ).unwrap();
    }
}

fn watch_json(
    watches: &Rc<RefCell<HashMap<PathBuf, JsFunction>>>,
    _this: &JsValue, args: &[JsValue], context: &mut Context
    ) -> JsResult<JsValue>
{
    if args.len() < 2 {
        return Err(JsNativeError::typ().with_message("Not enough arguments").into());
    }
    
    let app_path = context.global_object().get("app_path", context)?
        .try_js_into::<String>(context)?;
    let mut full_path = PathBuf::from(app_path);
    
    let path = args[0].try_js_into::<String>(context)?;
    full_path.push(path);
    
    let callback = args[1].try_js_into::<JsFunction>(context)?;
    // todo Keeping the callback outside the JsEnv seems to cause core dump on quit
    watches.borrow_mut().insert(full_path.clone(), callback.clone());
    
    let run_first = match args.get(2) {
        Some(arg) => arg.try_js_into::<bool>(context)?,
        None => true,
    };
    
    if run_first {
        let data = JsEnv::load_json(&full_path, context)?;
        callback.call(&JsValue::Undefined, &[data], context)?;
    }
    
    JsEnv::load_json(&full_path, context)
}

impl JsEnv {
    pub fn handle_file_changes(&mut self) {
        let mut reload_script_env = false;
        
        for changed_path_buf in iter_unique(self.file_change_rx.try_iter()) {
            // Check if it's a watched file with a callback
            if let Some(js_fn) = self.watches.borrow().get(&changed_path_buf) {               
                match JsEnv::load_json(&changed_path_buf, &mut self.context) {
                    Ok(data) => {
                        if let Err(err) = js_fn.call(&JsValue::Undefined, &[data], &mut self.context) {
                            dbg!(&err);
                        }
                    },
                    Err(err) => {dbg!(&err);},
                }
            }
            
            // If not explicitly watched, do other updates
            let extension = changed_path_buf.extension().and_then(|ext| ext.to_str());            
            match extension {
                Some(ext) if ext == "js" => {
                    reload_script_env = true;
                },
                _ => {},
            }
        }
               
        if reload_script_env {
            self.try_before_reload();
        }
    }
    
    fn try_before_reload(&mut self) {
        match JsEnv::create_context(&self.app_path, &self.graphics_calls, &self.watches) {
            Ok((mut context, module)) => {
                match JsEnv::call_module_init(&module, &mut context) {
                    Ok(_) => {
                        self.context = context;
                        self.module = module;
                        println!("Reloaded script environment.");
                    },
                    Err(err) => { dbg!(&err); },
                };
            },
            Err(err) => { dbg!(&err); },
        };        
    }
}