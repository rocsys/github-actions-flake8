use regex::RegexBuilder;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    let child = Command::new("flake8")
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn flake8 process!");

    let flake8_stdout = BufReader::new(child.stdout.unwrap());

    let stdout = &mut std::io::stdout().lock();

    let regex = RegexBuilder::new(r"^([^:]+):(\d+):(\d+): (.*)$")
        .build()
        .expect("Failed to build regex!");

    for line in flake8_stdout.lines().map(|line| line.unwrap()) {
        writeln!(stdout, "{}", &line).unwrap();
        if let Some(captures) = regex.captures(&line) {
            let path = &captures[1];
            let line = &captures[2];
            let column = &captures[3];
            let message = &captures[4];
            writeln!(
                stdout,
                "::warning file={path},line={line},col={column}::{message}"
            )
            .unwrap();
        }
    }

    stdout.flush().unwrap();
}
