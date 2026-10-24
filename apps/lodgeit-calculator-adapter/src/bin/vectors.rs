//! The differential vector runner: JSON cases on stdin, results on stdout.
fn main() -> std::io::Result<()> {
    lodgeit_calculator_adapter::vectors::run(
        std::io::stdin().lock(),
        std::io::BufWriter::new(std::io::stdout().lock()),
    )
}
