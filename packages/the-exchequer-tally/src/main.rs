use std::io::{Read, Write};
use the_exchequer_tally::{api, cli};

fn emit(mut stream: impl Write, text: &str, stderr: bool, json: bool) -> std::io::Result<()> {
    if !json {
        return stream.write_all(&cli::output_bytes(text, stderr));
    }
    // API text comes from JSON, never from an OS string, so it carries no surrogate
    // escapes to decode: write it as UTF-8 with only the native newline.
    if cfg!(windows) {
        stream.write_all(text.replace('\n', "\r\n").as_bytes())
    } else {
        stream.write_all(text.as_bytes())
    }
}
fn api_run(command: &str) -> (i32, String, String) {
    let mut bytes = Vec::new();
    // Bound the JSON boundary before allocating the complete input. The engine never opens paths.
    if let Err(e) = std::io::stdin().take((8 << 20) + 1).read_to_end(&mut bytes) {
        return (2, String::new(), format!("error: {e}\n"));
    }
    let result = if bytes.len() > 8 << 20 {
        Err("JSON input exceeds 8 MiB".into())
    } else {
        serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid JSON: {e}"))
            .and_then(|v| api::evaluate(command, &v))
    };
    match result {
        Ok(v) => (0, format!("{v}\n"), String::new()),
        Err(m) => (2, String::new(), format!("error: {m}\n")),
    }
}
fn main() {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| cli::decode_os(&a))
        .collect();
    let json = args.len() == 1 && api::COMMANDS.contains(&args[0].as_str());
    let (code, out, err) = if json {
        api_run(&args[0])
    } else if args == ["--api-help"] {
        (
            0,
            format!(
                "JSON stdin commands: {}\nMoney, rates and percentages are decimal strings. See README.md for schemas.\n",
                api::COMMANDS.join(", ")
            ),
            String::new(),
        )
    } else {
        cli::run(&args)
    };
    let out_result = emit(std::io::stdout().lock(), &out, false, json);
    let err_result = emit(std::io::stderr().lock(), &err, true, json);
    std::process::exit(if out_result.is_err() || err_result.is_err() {
        1
    } else {
        code
    });
}
