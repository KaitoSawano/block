use zenachain::cli;

fn main() {
    println!("[main] ZenaChain node dimulai.");
    let args: Vec<String> = std::env::args().collect();
    cli::run(&args);
}
