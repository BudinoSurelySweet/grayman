#[macro_export]
macro_rules! log_type {
    (info) => {
        "[grayman] info".bright_blue().bold()
    };
    (warn) => {
        "[grayman] warn".bright_yellow().bold()
    };
    (error) => {
        "[grayman] error".bright_red().bold()
    };
}

#[macro_export]
macro_rules! make_log {
	($log_type:ident, $($arg:tt)*) => {
        {
		    use colored::Colorize;

		    format!("{:<18} {}", $crate::log_type!($log_type), format_args!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! log {
    ($log_type:ident, $($arg:tt)*) => {
        println!("{}", $crate::make_log!($log_type, $($arg)*))
    };
}
