fn main() {
    // `verify` and `contract` walk untrusted JSON recursively, as Python's C
    // encoder does, so the commands run on a large stack.
    let child = std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| lodgeit_calculator_adapter::cli::main(std::env::args_os().skip(1).collect()))
        .expect("spawn the main thread");
    let code = child.join().unwrap_or(1);
    std::process::exit(code);
}
