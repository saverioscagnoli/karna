mod bundle;
mod jstypes;
mod run;

use std::path::PathBuf;

enum Command {
    Bundle { path: PathBuf },
    Run { path: PathBuf },
    JSTypes { path: Option<PathBuf> },
}

struct Args {
    command: Command,
}

impl Args {
    fn parse_path(parser: &mut lexopt::Parser) -> Result<PathBuf, lexopt::Error> {
        use lexopt::prelude::*;

        let mut path = None;

        while let Some(arg) = parser.next()? {
            match arg {
                Value(val) if path.is_none() => path = Some(PathBuf::from(val)),
                _ => return Err(arg.unexpected()),
            }
        }

        path.ok_or_else(|| "missing <path>".into())
    }

    fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_env();

        let subcommand = loop {
            match parser.next()? {
                Some(Short('h') | Long("help")) => {
                    println!("Help");
                    std::process::exit(0);
                }

                Some(Value(val)) => break val.string()?,
                Some(arg) => return Err(arg.unexpected()),
                None => return Err("missing subcommand".into()),
            }
        };

        let command = match subcommand.as_str() {
            "bundle" => Command::Bundle {
                path: Self::parse_path(&mut parser)?,
            },
            "run" => Command::Run {
                path: Self::parse_path(&mut parser)?,
            },
            "jstypes" => Command::JSTypes {
                path: Self::parse_path(&mut parser).ok(),
            },
            other => return Err(format!("unknown subcommand '{other}'").into()),
        };

        Ok(Self { command })
    }
}

fn main() {
    if let Some(archive) = bundle::embedded() {
        nostd::fs::mount(archive);

        if let Err(e) = run::exec(&PathBuf::from(".")) {
            eprintln!("error: {e}");
        }

        return;
    }

    let command = match Args::parse() {
        Ok(args) => args.command,
        Err(e) => {
            eprintln!("error: {e}");
            return;
        }
    };

    match command {
        Command::Bundle { path } => {
            if let Err(e) = bundle::exec(&path) {
                eprintln!("error: {e}");
            }
        }

        Command::Run { path } => {
            if let Err(e) = run::exec(&path) {
                eprintln!("error: {e}");
            }
        }

        Command::JSTypes { path } => {
            if let Err(e) = jstypes::exec(path.as_ref()) {
                eprintln!("error: {e}");
            }
        }
    }
}
