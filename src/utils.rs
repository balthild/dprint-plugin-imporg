use std::fmt::Debug;
use std::io::Write;

use dprint_core::plugins::wasm::WasiPrintFd;

#[allow(unused)]
pub fn debug_print<T: Debug>(value: T) -> T {
    let formatted = format!("{:#?}", value);
    let _ = WasiPrintFd(2).write_all(formatted.as_bytes());
    value
}

#[allow(unused)]
pub fn log(message: &str) {
    let _ = WasiPrintFd(1).write_all(message.as_bytes());
}

#[macro_export]
macro_rules! re {
    ($re:expr) => {{
        static RE: ::std::sync::LazyLock<::regex::Regex> =
            ::std::sync::LazyLock::new(|| ::regex::Regex::new($re).unwrap());
        &RE
    }};
}
