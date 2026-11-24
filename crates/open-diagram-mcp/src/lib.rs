//! Stdio MCP server for open-diagram-rs.
//!
//! [`DiagramServer`] serves `format_guide`, `template`, `check`, `describe`, and `draw`.

mod guide;
mod server;
mod work;

pub use server::DiagramServer;
