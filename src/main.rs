use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
};

use github_actions_flake8::flake8;

use clap::Parser;
use log::debug;

const LONG_ABOUT: &str = r"
Takes flake8 output through stdin or launches flake8 with the provided optional arguments and transforms the lint messages into Github Actions instructions.
";

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = LONG_ABOUT)]
struct Options {
    /// Level at which to emit Github Actions messages.
    #[clap(long, value_enum, default_value = "error")]
    level: Level,

    /// If nothing is piped through stdin, this program will attempt to spawn flake8 with these arguments.
    #[clap(last = true)]
    flake8_args: Vec<String>,
}

#[derive(Debug, Copy, clap::ValueEnum, Clone)]
enum Level {
    Warning,
    Error,
}

fn spawn_flake8(args: &[String]) -> Child {
    Command::new("flake8")
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn flake8 process!")
}

fn main() {
    env_logger::builder().init();

    let options = Options::parse();

    let mut output = std::io::stdout().lock();

    let statistics = if atty::is(atty::Stream::Stdin) {
        debug!("Spawning flake8...");

        let mut flake8 = spawn_flake8(&options.flake8_args);
        let mut input = BufReader::new(flake8.stdout.take().unwrap());

        options.rewrite(&mut input, &mut output).unwrap()
    } else {
        debug!("Reading from stdin...");

        let mut input = std::io::stdin().lock();

        options.rewrite(&mut input, &mut output).unwrap()
    };

    output.flush().unwrap();

    debug!("Processed {} lines.", statistics.line_count);
    debug!("Rewrote {} lines.", statistics.rewrite_count);

    std::process::exit(if statistics.rewrite_count > 0 { 1 } else { 0 });
}

#[derive(Default)]
struct Statistics {
    line_count: u64,
    rewrite_count: u64,
}

impl Options {
    fn rewrite<R: BufRead, W: Write>(&self, reader: R, mut writer: W) -> std::io::Result<Statistics> {
        let mut rewriter = Rewriter::default();
        let mut statistics = Statistics::default();

        for line in reader.lines().map(|line| line.unwrap()) {
            if rewriter
                .rewrite_line(&mut writer, &line, self.level)
                .unwrap()
            {
                statistics.rewrite_count += 1;
            }
            statistics.line_count += 1;
        }

        Ok(statistics)
    }
}

#[derive(Default)]
struct Rewriter {
    parser: flake8::LineParser,
}

impl Rewriter {
    fn rewrite_line<W: Write>(
        &mut self,
        mut writer: W,
        line: &str,
        level: Level,
    ) -> std::io::Result<bool> {
        let level = match level {
            Level::Error => "error",
            Level::Warning => "warning",
        };

        let rewritten = match self.parser.parse(line) {
            Some(flake8::LineMatch {
                path,
                line,
                column,
                message,
            }) => {
                writeln!(
                    writer,
                    "::{level} file={path},line={line},col={column}::{message}"
                )?;
                true
            }
            None => {
                writeln!(writer, "{}", line)?;
                false
            }
        };

        Ok(rewritten)
    }
}
