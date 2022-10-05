#[macro_export]
macro_rules! error {
    ($fmt_str:literal) => {
        console::style(format!($fmt_str)).bold().red()
    };
    ($fmt_str:literal, $($arg:tt)*) => {
        console::style(format!($fmt_str, $($arg)*)).bold().red()
    };
}

#[macro_export]
macro_rules! info {
    ($fmt_str:literal) => {
        console::style(format!($fmt_str)).bold().cyan()
    };
    ($fmt_str:literal, $($arg:tt)*) => {
        console::style(format!($fmt_str, $($arg)*)).bold().cyan()
    };
}

#[macro_export]
macro_rules! message {
    ($fmt_str:literal) => {
        console::style(format!($fmt_str)).bold().green()
    };
    ($fmt_str:literal, $($arg:tt)*) => {
        console::style(format!($fmt_str, $($arg)*)).bold().green()
    };
}

#[macro_export]
macro_rules! warning {
    ($fmt_str:literal) => {
        console::style(format!($fmt_str)).bold().yellow()
    };
    ($fmt_str:literal, $($arg:tt)*) => {
        console::style(format!($fmt_str, $($arg)*)).bold().yellow()
    };
}
