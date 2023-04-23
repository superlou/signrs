use std::path::Path;
use std::fs::read_to_string;

use chrono::DateTime;
use rhai::{Engine, Scope, AST, CallFnOptions, FnPtr, EvalAltResult, FuncArgs, RegisterNativeFunction, Map};
use thiserror::Error;
use rhai::{Variant, Identifier};

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("EvalAltError: {0}")]
    EvalAltError(#[from] Box<EvalAltResult>),
    #[error("FileReadError")]
    FileReadError(#[from] std::io::Error),
    #[error("JsonParseError: {0}")]
    JsonParseError(Box<EvalAltResult>),
}

pub struct ScriptEnv {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
}

impl ScriptEnv {
    pub fn new(main: &Path) -> Self {
        let engine = Engine::new();
        let ast = engine.compile_file(main.to_owned()).unwrap();
        
        ScriptEnv {
            engine, ast,
            scope: Scope::new(),
        }
    }
    
    fn register_str_formatting(&mut self) {
        self.engine.register_fn("str", |f: f32| {
           format!("{0}", f)
        });
        
        self.engine.register_fn("str", |f: f32, precision: i64| {
           format!("{0:.1$}", f, precision as usize)
        });
        
        self.engine.register_fn("str", |dt: DateTime<chrono::Utc>| {
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        });
        
        self.engine.register_fn("str", |dt: DateTime<chrono::Utc>, fmt: &str, as_local: bool| {
            if as_local {
                dt.with_timezone(&chrono::Local).format(fmt).to_string()
            } else {
                dt.format(fmt).to_string()
            }
        });        
    }
    
    fn register_datetime(&mut self) {
        self.engine.register_type_with_name::<DateTime<chrono::Utc>>("DateTime")
            .register_fn("now", || chrono::Utc::now());
    }
    
    pub fn eval_initial(&mut self) -> Result<(), ScriptError> {
        self.register_str_formatting();
        self.register_datetime();
        
        match self.engine.eval_ast_with_scope::<()>(&mut self.scope, &self.ast) {
            Ok(_) => Ok(()),
            Err(e) => Err(ScriptError::EvalAltError(e))
        }
    }
    
    pub fn hotload_rhai(&mut self, path: &Path) -> Result<(), ScriptError> {
        let new_ast = self.engine.compile_file_with_scope(&self.scope, path.to_owned())?;
        self.ast.combine(new_ast);
        self.engine.eval_ast_with_scope::<()>(&mut self.scope, &self.ast)?;
        Ok(())
    }
    
    pub fn register_fn<A: 'static, const N: usize, const C: bool, R: Variant + Clone, const L: bool,
                       F: RegisterNativeFunction<A, N, C, R, L>>(
        &mut self, name: impl AsRef<str> + Into<Identifier>, func: F
    ) {
        self.engine.register_fn(name, func);
    }
    
    pub fn register_type<T: Variant + Clone>(&mut self, name: &str) -> &mut Self {
        self.engine.register_type_with_name::<T>(name);
        self
    }
    
    pub fn get_value<T: Variant + Clone>(&self, name: &str) -> Option<T> {
        self.scope.get_value::<T>(name)
    }
    
    pub fn call_fn_ptr<T>(&self, fn_ptr: &FnPtr, args: impl FuncArgs) -> Result<T, ScriptError>
    where T: Variant + Clone
    {
        match fn_ptr.call::<T>(&self.engine, &self.ast, args) {
            Ok(ret) => Ok(ret),
            Err(e) => Err(ScriptError::EvalAltError(e)),
        }
    }
    
    pub fn call_fn<T>(&mut self, name: &str, args: impl FuncArgs) -> Result<T, ScriptError>
    where T: Variant + Clone
    {
        let options = CallFnOptions::new().eval_ast(false);
        match self.engine.call_fn_with_options::<T>(
            options,
            &mut self.scope,
            &mut self.ast,
            name,
            args
        ) {
            Ok(ret) => Ok(ret),
            Err(e) => Err(ScriptError::EvalAltError(e))
        }
    }
        
    pub fn parse_json_file(&self, path: &Path) -> Result<Map, ScriptError> {
        let json_text = match read_to_string(&path) {
            Ok(s) => s,
            Err(e) => return Err(ScriptError::FileReadError(e)),
        };
        
        match self.engine.parse_json(json_text, true) {
            Ok(data) => Ok(data),
            Err(e) => {
                 Err(ScriptError::JsonParseError(e))
            }
        }
    }
}