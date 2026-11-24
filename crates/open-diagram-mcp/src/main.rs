use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use open_diagram_mcp::DiagramServer;

#[derive(Parser)]
#[command(
    name = "open-diagram-mcp",
    disable_version_flag = true,
    about = "Run open-diagram as an MCP server over stdio (started by an MCP app)."
)]
struct Cli {
    /// Folder for SVG and PNG files (default: `$OPEN_DIAGRAM_OUT`, else `./open-diagram`).
    #[arg(long)]
    out: Option<PathBuf>,
    /// A folder of icons diagrams may use (default: `$OPEN_DIAGRAM_ICONS`). The built-in icons are always included.
    #[arg(long, action = clap::ArgAction::Append)]
    icons: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("open-diagram {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let cli = Cli::parse();
    let server = DiagramServer::new(cli.out, &cli.icons);
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::from(1);
        }
    };
    match runtime.block_on(server.serve_stdio()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}
