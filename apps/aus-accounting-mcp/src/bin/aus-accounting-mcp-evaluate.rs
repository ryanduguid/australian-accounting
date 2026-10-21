//! Offline supplementary evaluation; recordings are never executed.
#[path = "../evaluation/mod.rs"]
mod evaluation;

#[tokio::main]
async fn main() {
    std::process::exit(evaluation::main(std::env::args_os().skip(1).collect()).await);
}
