wit_bindgen::generate!({
    world: "security-tool",
    path: "wit",
});

struct HelloWorldPlugin;

impl Guest for HelloWorldPlugin {
    fn execute(target: String) -> Result<PluginReport, String> {
        Ok(PluginReport {
            plugin_name: "Hello World Plugin".to_string(),
            target_tested: target,
            issues: vec![],
            summary: "Hello World from PolyGlid guest runtime component!".to_string(),
        })
    }

    fn metadata() -> PluginMetadata {
        PluginMetadata {
            name: "hello_world".to_string(),
            display_name: "Hello World".to_string(),
            version: "0.1.0".to_string(),
            description: "Hello World demo plugin".to_string(),
            author: "PolyGlid Team".to_string(),
        }
    }

    fn required_capabilities() -> Vec<String> {
        vec![]
    }

    fn cli_panel(_report: PluginReport) -> PanelLayout {
        PanelLayout {
            title: "Hello World Layout".to_string(),
            widgets: vec![],
        }
    }

    fn desktop_panel(report: PluginReport) -> PanelLayout {
        Self::cli_panel(report)
    }
}

export!(HelloWorldPlugin);
