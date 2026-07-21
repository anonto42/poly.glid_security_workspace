use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Vec<ToolParam>,
}

#[derive(Debug, Clone)]
pub struct ToolParam {
    pub name: &'static str,
    pub param_type: &'static str,
    pub description: &'static str,
    pub required: bool,
}

pub struct ToolRegistry {
    pub tools: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![
                ToolDefinition {
                    name: "read_file",
                    description: "Read the content of a file, optionally specific lines",
                    parameters: vec![
                        ToolParam { name: "path", param_type: "string", description: "File path relative to workspace root", required: true },
                        ToolParam { name: "start_line", param_type: "integer", description: "First line number (1-indexed)", required: false },
                        ToolParam { name: "end_line", param_type: "integer", description: "Last line number (inclusive)", required: false },
                    ],
                },
                ToolDefinition {
                    name: "search_code",
                    description: "Search codebase for a pattern (grep)",
                    parameters: vec![
                        ToolParam { name: "pattern", param_type: "string", description: "Regex or text to search for", required: true },
                        ToolParam { name: "path_filter", param_type: "string", description: "Only search files matching this glob", required: false },
                    ],
                },
                ToolDefinition {
                    name: "list_files",
                    description: "List files in a directory",
                    parameters: vec![
                        ToolParam { name: "path", param_type: "string", description: "Directory path relative to workspace root", required: true },
                        ToolParam { name: "recursive", param_type: "boolean", description: "List recursively", required: false },
                    ],
                },
                ToolDefinition {
                    name: "run_test",
                    description: "Run a specific test or test file",
                    parameters: vec![
                        ToolParam { name: "target", param_type: "string", description: "Test name or path", required: true },
                        ToolParam { name: "language", param_type: "string", description: "rust|node", required: false },
                    ],
                },
                ToolDefinition {
                    name: "read_git_log",
                    description: "Read recent git commit history",
                    parameters: vec![
                        ToolParam { name: "limit", param_type: "integer", description: "Number of commits to show", required: false },
                    ],
                },
            ],
        }
    }

    pub fn describe_all(&self) -> String {
        self.tools.iter().map(|t| {
            let params: Vec<String> = t.parameters.iter().map(|p| {
                format!("    - {} ({}): {}{}", p.name, p.param_type, p.description,
                    if p.required { " [required]" } else { " [optional]" })
            }).collect();
            format!("  - {}\n    {}\n{}", t.name, t.description, params.join("\n"))
        }).collect::<Vec<_>>().join("\n")
    }
}
