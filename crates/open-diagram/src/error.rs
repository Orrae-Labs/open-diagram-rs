use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramError {
    message: String,
}

impl DiagramError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DiagramError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DiagramError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecError {
    message: String,
}

impl SpecError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SpecError {}

impl From<DiagramError> for SpecError {
    fn from(err: DiagramError) -> Self {
        Self::new(err.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MermaidError {
    message: String,
}

impl MermaidError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for MermaidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MermaidError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutError {
    pub problems: Vec<String>,
    pub files: Vec<String>,
}

impl LayoutError {
    #[must_use]
    pub const fn new(problems: Vec<String>, files: Vec<String>) -> Self {
        Self { problems, files }
    }
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = format!("{} layout problem(s):", self.problems.len());
        for problem in &self.problems {
            message.push('\n');
            message.push_str("  - ");
            message.push_str(problem);
        }
        if !self.files.is_empty() {
            message.push('\n');
            message.push_str("The drawing was still written, so you can look at it: ");
            message.push_str(&self.files.join(", "));
        }
        f.write_str(&message)
    }
}

impl std::error::Error for LayoutError {}

#[derive(Debug)]
pub enum SaveError {
    Diagram(DiagramError),
    Layout(LayoutError),
    Io(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagram(err) => f.write_str(err.message()),
            Self::Layout(err) => err.fmt(f),
            Self::Io(err) => f.write_str(err),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<DiagramError> for SaveError {
    fn from(err: DiagramError) -> Self {
        Self::Diagram(err)
    }
}
