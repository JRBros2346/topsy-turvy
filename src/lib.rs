use serde::Deserialize;

mod code;
mod submit;

pub use submit::{handle_submit, Output};

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Cpp,
    Javascript,
    Python,
    Java,
}
impl Language {
    pub fn is_compiled(&self) -> bool {
        use Language::*;
        matches!(self, Rust | Cpp | Java)
    }
}