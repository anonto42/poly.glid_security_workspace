wit_bindgen::generate!({
    world: "security-tool",
    path: "wit",
});

struct TemplatePlugin;

impl Guest for TemplatePlugin {
    fn execute(target: String) -> Result<PluginReport, String> {
        Ok(PluginReport {
            plugin_name: "Template Plugin".to_string(),
            target_tested: target,
            issues: vec![],
            summary: "Template executed successfully.".to_string(),
        })
    }

    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "template_plugin".to_string(),
            display_name: "Template Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A skeleton helper plugin".to_string(),
            author: "Author".to_string(),
        }
    }

    fn required_capabilities() -> Vec<String> {
        vec![]
    }

    fn cli_panel(_report: PluginReport) -> PanelLayout {
        PanelLayout {
            title: "Template Panel".to_string(),
            widgets: vec![],
        }
    }

    fn desktop_panel(report: PluginReport) -> PanelLayout {
        Self::cli_panel(report)
    }
}

export!(TemplatePlugin);
