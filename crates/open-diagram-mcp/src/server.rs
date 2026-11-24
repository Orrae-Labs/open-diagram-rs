use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use rmcp::handler::server::skill_catalog::SkillCatalog;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, GetSkillRequestParams, GetSkillResult, Implementation,
    ListResourcesResult, ListSkillsResult, PaginatedRequestParams,
    ReadResourceDirectoryRequestParams, ReadResourceDirectoryResult, ReadResourceRequestParams,
    ReadResourceResponse, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::guide::INSTRUCTIONS;
use crate::work::{self, absolute_user, default_out, icon_folders};

const FORMAT_GUIDE: &str = "Read this once before writing an open-diagram-rs diagram. It lists the JSON keys, the Mermaid subset, and how to fix each layout problem. It does not check or draw; use check, then draw.";
const TEMPLATE_DESC: &str = "List the open-diagram-rs starter diagrams, or return one named starter as JSON. Use it to start. Use check to validate and draw to write files. An empty name lists the starters.";
const CHECK_DESC: &str = "Build an open-diagram-rs diagram in memory and list its layout problems, without writing files. A problem list is the answer: fix each named part and call check again until it reports 0. Use describe for the words and draw for the files. format is json, mermaid, or auto.";
const DESCRIBE_DESC: &str = "Say what an open-diagram-rs diagram contains, in plain words: blocks, states, groups, lines, and the key. It does not check layout and does not write files. Use check for problems and draw for the picture.";
const DRAW_DESC: &str = "Write an open-diagram-rs SVG and PNG into the output folder. Returns the layout problems, the file paths, a plain-words description, and the picture when picture is true. Call check first until it reports 0. This is the only tool that writes files, and only inside the output folder.";

fn auto_format() -> String {
    "auto".to_owned()
}

fn diagram_name() -> String {
    "diagram".to_owned()
}

const fn picture_on() -> bool {
    true
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TemplateArgs {
    /// Template name. Leave it empty to list the templates.
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SourceArgs {
    /// Diagram as JSON or Mermaid text.
    source: String,
    /// `json`, `mermaid`, or `auto`.
    #[serde(default = "auto_format")]
    format: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DrawArgs {
    /// Diagram as JSON or Mermaid text.
    source: String,
    /// `json`, `mermaid`, or `auto`.
    #[serde(default = "auto_format")]
    format: String,
    /// File name inside the output folder.
    #[serde(default = "diagram_name")]
    name: String,
    /// Include the PNG in the tool result.
    #[serde(default = "picture_on")]
    picture: bool,
}

/// Stdio MCP server. It draws JSON and Mermaid, and it writes files only inside one output folder.
#[derive(Clone, Debug)]
pub struct DiagramServer {
    out_dir: PathBuf,
    icons: Vec<PathBuf>,
    skills: Option<Arc<SkillCatalog>>,
}

impl DiagramServer {
    /// `out` defaults to `$OPEN_DIAGRAM_RS_OUT`, then `./open-diagram-rs` when that directory is writable.
    ///
    /// An empty `icons` list reads `$OPEN_DIAGRAM_RS_ICONS`. The built-in icons are always included.
    #[must_use]
    pub fn new(out: Option<PathBuf>, icons: &[PathBuf]) -> Self {
        Self {
            out_dir: out.map_or_else(default_out, absolute_user),
            icons: icon_folders(icons),
            skills: load_skills(),
        }
    }

    /// Serve MCP on stdin and stdout until the client closes the stream.
    ///
    /// # Errors
    /// Returns an error when the stdio transport stops with a protocol failure.
    pub async fn serve_stdio(self) -> Result<(), String> {
        let running = self
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|err| err.to_string())?;
        running.waiting().await.map_err(|err| err.to_string())?;
        Ok(())
    }
}

fn load_skills() -> Option<Arc<SkillCatalog>> {
    let dir = find_skills_dir()?;
    match SkillCatalog::from_dir(&dir) {
        Ok(catalog) => Some(Arc::new(catalog)),
        Err(err) => {
            eprintln!(
                "open-diagram-rs-mcp: skills folder {}: {err}",
                dir.display()
            );
            None
        }
    }
}

fn find_skills_dir() -> Option<PathBuf> {
    if let Some(value) = std::env::var_os("OPEN_DIAGRAM_RS_SKILLS") {
        let path = absolute_user(PathBuf::from(value));
        if path.is_dir() {
            return Some(path);
        }
        eprintln!(
            "open-diagram-rs-mcp: OPEN_DIAGRAM_RS_SKILLS is not a folder ({})",
            path.display()
        );
    }
    let mut starts = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        starts.push(parent.to_path_buf());
    }
    starts.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    for start in starts {
        let mut dir = start;
        for _ in 0..8 {
            let nested = dir.join("skills");
            if nested.join("open-diagram-rs").join("SKILL.md").is_file() {
                return Some(nested);
            }
            if dir.join("open-diagram-rs").join("SKILL.md").is_file() {
                return Some(dir);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    None
}

fn failure(message: impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.to_string())])
}

fn success(text: String) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(text)])
}

#[tool_router]
impl DiagramServer {
    #[tool(
        description = FORMAT_GUIDE,
        title = "open-diagram-rs format guide",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    fn format_guide(&self) -> CallToolResult {
        success(work::icons_guide(&self.icons))
    }

    #[tool(
        description = TEMPLATE_DESC,
        title = "open-diagram-rs template",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    #[allow(
        clippy::unused_self,
        reason = "the tool macro only accepts methods, and listing templates does not use the server"
    )]
    fn template(
        &self,
        Parameters(TemplateArgs { name }): Parameters<TemplateArgs>,
    ) -> CallToolResult {
        work::template_text(&name).map_or_else(failure, success)
    }

    #[tool(
        description = CHECK_DESC,
        title = "open-diagram-rs check",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn check(
        &self,
        Parameters(SourceArgs { source, format }): Parameters<SourceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let icons = self.icons.clone();
        match tokio::task::spawn_blocking(move || work::check_text(&source, &format, &icons)).await
        {
            Ok(Ok(text)) => Ok(success(text)),
            Ok(Err(err)) => Ok(failure(err)),
            Err(err) => Ok(failure(err)),
        }
    }

    #[tool(
        description = DESCRIBE_DESC,
        title = "open-diagram-rs describe",
        annotations(read_only_hint = true, open_world_hint = false)
    )]
    async fn describe(
        &self,
        Parameters(SourceArgs { source, format }): Parameters<SourceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let icons = self.icons.clone();
        match tokio::task::spawn_blocking(move || work::describe_text(&source, &format, &icons))
            .await
        {
            Ok(Ok(text)) => Ok(success(text)),
            Ok(Err(err)) => Ok(failure(err)),
            Err(err) => Ok(failure(err)),
        }
    }

    #[tool(
        description = DRAW_DESC,
        title = "open-diagram-rs draw",
        annotations(read_only_hint = false, destructive_hint = false, open_world_hint = false)
    )]
    async fn draw(
        &self,
        Parameters(DrawArgs {
            source,
            format,
            name,
            picture,
        }): Parameters<DrawArgs>,
    ) -> Result<CallToolResult, McpError> {
        let icons = self.icons.clone();
        let out_dir = self.out_dir.clone();
        let joined = tokio::task::spawn_blocking(move || {
            work::draw_files(&source, &format, &name, &out_dir, picture, &icons)
        })
        .await;
        let drawn = match joined {
            Ok(Ok(drawn)) => drawn,
            Ok(Err(err)) => return Ok(failure(err)),
            Err(err) => return Ok(failure(err)),
        };
        let mut blocks = vec![ContentBlock::text(drawn.text)];
        if let Some(png) = drawn.png {
            blocks.push(ContentBlock::image(
                base64::engine::general_purpose::STANDARD.encode(png),
                "image/png",
            ));
        }
        Ok(CallToolResult::success(blocks))
    }
}

#[tool_handler]
#[allow(
    clippy::unused_async_trait_impl,
    reason = "ServerHandler and the tool_handler macro are async even when a method does not await"
)]
impl ServerHandler for DiagramServer {
    fn get_info(&self) -> ServerConfig {
        let capabilities = if self.skills.is_some() {
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_skills_with_directory_read()
                .build()
        } else {
            ServerCapabilities::builder().enable_tools().build()
        };
        let info = Implementation::new("open-diagram-rs", env!("CARGO_PKG_VERSION"))
            .with_website_url("https://github.com/Orrae-Labs/open-diagram-rs");
        ServerConfig::new(capabilities)
            .with_server_info(info)
            .with_instructions(INSTRUCTIONS.to_owned())
    }

    async fn list_skills(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListSkillsResult, McpError> {
        Ok(self.skills.as_ref().map_or_else(
            || ListSkillsResult::with_all_items(Vec::new()),
            |catalog| catalog.list_skills(),
        ))
    }

    async fn get_skill(
        &self,
        request: GetSkillRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetSkillResult, McpError> {
        let Some(catalog) = &self.skills else {
            return Err(McpError::invalid_params(
                "this server has no skills folder",
                None,
            ));
        };
        catalog.get_skill(&request.uri)
    }

    async fn read_resource_directory(
        &self,
        request: ReadResourceDirectoryRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceDirectoryResult, McpError> {
        let Some(catalog) = &self.skills else {
            return Err(McpError::invalid_params(
                "this server has no skills folder",
                None,
            ));
        };
        catalog.read_resource_directory(&request.uri)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(
            self.skills
                .as_ref()
                .map_or_default(|catalog| catalog.resources()),
        ))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let Some(catalog) = &self.skills else {
            return Err(McpError::resource_not_found(request.uri, None));
        };
        catalog
            .read_resource(&request.uri)
            .map(Into::into)
            .ok_or_else(|| McpError::resource_not_found(request.uri, None))
    }
}
