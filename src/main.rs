use zenachain::cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    cli::run(&args);
}
