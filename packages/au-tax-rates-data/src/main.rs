fn main() {
    // A record's pattern is matched by a backtracking matcher that recurses
    // once per repeat iteration, so the commands run on a large stack.
    let child = std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| au_tax_rates_data::rates::main(std::env::args_os().skip(1).collect()))
        .expect("spawn the main thread");
    let code = child.join().unwrap_or(1);
    std::process::exit(code);
}
