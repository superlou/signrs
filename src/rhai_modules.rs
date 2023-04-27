use chrono::DateTime;
use rhai::plugin::*;

#[export_module]
pub mod str {
    pub fn str(f: f32) -> String {
        format!("{0}", f)
    }
    
    #[rhai_fn(name="str")]
    pub fn str_with_precision(f: f32, precision: i64) -> String {
        format!("{0:.1$}", f, precision as usize)
    }
    
    #[rhai_fn(name="str")]
    pub fn str_utc(dt: DateTime<chrono::Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    #[rhai_fn(name="str")]
    pub fn str_fmt(dt: DateTime<chrono::Utc>, fmt: &str, as_local: bool) -> String {
        if as_local {
            dt.with_timezone(&chrono::Local).format(fmt).to_string()
        } else {
            dt.format(fmt).to_string()
        }
    }
}

#[export_module]
pub mod datetime {
    pub type DateTime = chrono::DateTime<chrono::Utc>;
    
    pub fn now() -> DateTime {
        chrono::Utc::now()
    }
}