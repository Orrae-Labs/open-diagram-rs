use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: quickstart-consumer OUT.svg");
        return ExitCode::from(2);
    };
    let mut diagram = match open_diagram::examples::quickstart() {
        Ok(diagram) => diagram,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::from(2);
        }
    };
    let report = match diagram.render() {
        Ok(report) => report,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::from(2);
        }
    };
    if let Err(err) = std::fs::write(&path, report.svg.as_bytes()) {
        eprintln!("error: {path}: {err}");
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}
