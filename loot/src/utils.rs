#[macro_export]
macro_rules! print_debug {
    ($cli:expr, $($arg:tt)*) => {
        if $cli.debug {
            let to_print = format!($($arg)*);
            println!("{color_yellow}{}{color_reset}", to_print);
        }
    };
}
