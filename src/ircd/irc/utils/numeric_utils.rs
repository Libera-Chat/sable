#[macro_export]
macro_rules! make_numeric {
    ($type:ident) => {
        crate::ircd::irc::messages::numeric::$type::new()
    };
    ($type:ident, $($args:expr),*) => {
        crate::ircd::irc::messages::numeric::$type::new($($args),*)
    };
}

#[macro_export]
macro_rules! numeric_error {
    ($($args:tt)*) => {
        Err(crate::make_numeric!( $($args)* ).into())
    }
}

pub use make_numeric;
pub use numeric_error;
