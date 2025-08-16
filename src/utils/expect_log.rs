use tracing::{Level, event};

#[allow(unused)]
pub trait ExpectLog<T> {
    fn expect_log(self, message: impl AsRef<str>) -> T;
}

impl<T> ExpectLog<T> for Option<T> {
    #[inline]
    fn expect_log(self, message: impl AsRef<str>) -> T {
        self.unwrap_or_else(|| {
            let message = message.as_ref();
            event!(Level::ERROR, "{}", message);
            panic!("{}", message);
        })
    }
}

impl<T, E> ExpectLog<T> for Result<T, E> {
    #[inline]
    fn expect_log(self, message: impl AsRef<str>) -> T {
        self.unwrap_or_else(|_| {
            let message = message.as_ref();
            event!(Level::ERROR, "{}", message);
            panic!("{}", message);
        })
    }
}

#[allow(unused)]
pub mod prelude {
    pub use super::ExpectLog as _;
}
