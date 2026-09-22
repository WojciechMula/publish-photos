mod expr;
mod parser;
mod token;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub use expr::Expr;

#[macro_export]
macro_rules! err {
    ($($arg:expr),*) => {
        Err(format!($($arg,)*).into())
    }
}
