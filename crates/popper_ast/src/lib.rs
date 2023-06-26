pub(crate) mod constant;
pub(crate) mod expr;
pub(crate) mod stmt;
pub(crate) mod span;
pub(crate) mod basic_stmt;
pub(crate) mod op;
pub(crate) mod types;

#[cfg(feature = "visitor")]
pub mod visitor;

pub use op::*;
pub use constant::*;
pub use expr::*;
pub use stmt::*;
pub use span::*;
pub use basic_stmt::*;
pub use types::*;


#[cfg(feature = "serde")]
pub fn get_ast_from_json_file(file: &str) -> Vec<Statement> {
    let file = std::fs::read_to_string(file).unwrap();
    let ast: Vec<Statement> = serde_json::from_str(&file).unwrap();
    ast
}

