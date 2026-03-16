use colored::Colorize;

pub fn report_error(filename: &str, line: usize, col: usize, message: &str) {
    eprintln!(
        "{}: {} at {}:{}:{}",
        "error".red().bold(),
        message,
        filename,
        line,
        col
    );
}

pub fn report_runtime_error(message: &str) {
    eprintln!("{}: {}", "runtime error".red().bold(), message);
}
