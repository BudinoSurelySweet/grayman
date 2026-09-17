#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        {
	        use colored::Colorize;

	        println!("{:<8} {}", "Info".bright_blue().bold(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! make_info {
    ($($arg:tt)*) => {
        {
	        use colored::Colorize;

	        format!("{:<8} {}", "Info".bright_blue().bold(), format_args!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        {
	        use colored::Colorize;

	        println!("{:<8} {}", "Warn".bright_yellow().bold(), format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        {
	        use colored::Colorize;

	        eprintln!("{:<8} {}", "Error".bright_red().bold(), format_args!($($arg)*));
        }
    };
}
