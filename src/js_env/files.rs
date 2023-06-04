use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use boa_engine::{Context, JsNativeError, JsResult, JsValue, NativeFunction};
use boa_engine::object::builtins::JsFunction;

use crate::js_env::JsEnv;

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
    
    let Ok(canonical_path) = fs::canonicalize(&full_path) else {return Ok(JsValue::Undefined)};
    let callback = args[1].try_js_into::<JsFunction>(context)?;
    // todo Keeping the callback outside the JsEnv seems to cause core dump on quit
    watches.borrow_mut().insert(canonical_path, callback.clone());
    
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