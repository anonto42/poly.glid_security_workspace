# Desktop User Interface Architecture

The PolyGlid desktop interface is built inside `apps/desktop` using **Tauri v2** and **React + TypeScript + TailwindCSS**. To support security operators, the UI is organized as a high-fidelity workspace mirroring a modern IDE layout (like VS Code).

## Layout Structure

```text
+--------------------------------------------------------------+
| A | Side Bar        | Editor Tab bar                         |
| c | (Explorer,      | [Scanner Dashboard] [recon_probe.rs]   |
| t |  Plugins, etc.) |----------------------------------------|
| i |                 |                                        |
| v |                 | Main Panel (scanner form, config inputs|
| i |                 | or read-only plugin code view)         |
| t |                 |                                        |
| y |                 |----------------------------------------|
|   |                 | Bottom Split Panel                     |
| B |                 | [PROBLEMS (0)] [OUTPUT] [TERMINAL]     |
| a |                 |                                        |
| r |                 | Results list, logs, or emulated shell  |
+---+-----------------+----------------------------------------+
| Status Bar (Core status, Wasmtime engine metrics, limits)   |
+--------------------------------------------------------------+
```

---

## UI Layout Components

All layouts live in `apps/desktop/src/components/layout/`.

### 1. `ActivityBar.tsx`
* **File Path:** [ActivityBar.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/ActivityBar.tsx)
* **Responsibility:** Leftmost vertical tab menu. Clicking icons changes the active sidebar panel or triggers modals.
* **Key Props:**
  * `activeView: string`: Tracks what view is selected in the sidebar.
  * `setActiveView: (view: string) => void`: Updates the sidebar selection.
  * `onSettingsClick: () => void`: Opens the global settings modal.

### 2. `SideBar.tsx`
* **File Path:** [SideBar.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/SideBar.tsx)
* **Responsibility:** Multi-view expandable side explorer.
  * **Explorer View:** Lists domains to scan (Targets) and lists active WebAssembly plugins. Includes forms to dynamically add/remove target hosts.
  * **Plugins View:** Shows loaded plugins and includes a form to load local plugins by typing their path.
  * **Settings View:** Displays inline configurators (like Wasmtime fuel limits).
* **Key Props:**
  * `targets: string[]`: Targets domains list.
  * `selectedTarget: string`: Current focused target domain.
  * `plugins: PluginInfo[]`: Array of registered WebAssembly plugins.
  * `onAddPlugin: (name: string, path: string) => void`: Callback to register a new WASM plugin path.

### 3. `EditorArea.tsx`
* **File Path:** [EditorArea.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/EditorArea.tsx)
* **Responsibility:** The main central display.
  * **Scanner Dashboard Tab:** Contains target inputs, plugin selection dropdowns, execution buttons, and errors.
  * **Source Viewer Tab (`recon_probe.rs`):** Displays read-only Rust code for active security plugins.
* **Key Props:**
  * `activeTab: string`: Which editor tab is currently visible (`dashboard` or `source`).
  * `setActiveTab: (tab: string) => void`: Switches tabs.
  * `plugins: PluginInfo[]`: Dynamically populates the plugin selection drop-down.
  * `onRunPlugin: (target: string) => Promise<void>`: Launches Wasmtime execution in the Tauri backend.

### 4. `BottomPanel.tsx`
* **File Path:** [BottomPanel.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/BottomPanel.tsx)
* **Responsibility:** Results display panel.
  * **Problems Tab:** Displays structured safety observation reports parsed from the plugin (using severity icons, titles, summaries, and recommendations).
  * **Output Tab:** Displays running engine logs and capability mappings.
  * **Terminal Tab:** Displays an emulated interactive host shell.
* **Key Props:**
  * `activeTab: string`: Active panel view (`problems`, `output`, `terminal`).
  * `report: Report | null`: The parsed observations object returned from the Rust sandbox.

### 5. `StatusBar.tsx`
* **File Path:** [StatusBar.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/StatusBar.tsx)
* **Responsibility:** Single horizontal strip at the bottom of the screen. Shows host connection statuses, Wasmtime status, and active fuel limits.

### 6. `SettingsModal.tsx`
* **File Path:** [SettingsModal.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/SettingsModal.tsx)
* **Responsibility:** Settings modal dialog. Opens as a backdrop overlay on top of the IDE. Contains tab segments for:
  * **System Overview:** Overview of Wasmtime integration, OS, and active permissions.
  * **Engine:** Editable inputs for fuel parameters.
  * **Plugins:** Details on loaded plugins.

---

## State Coordination

All workspace state lives in [App.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/App.tsx) and is passed down to components via standard React props:

1. **Target Selection Synchronization:** Clicking a target in the sidebar (`SideBar`) triggers `handleTargetSelect`, which updates the selected target state, changes the input field in the dashboard, and focuses the main editor back to the "Scanner Dashboard" tab.
2. **Auto-Focusing Observations:** When a user clicks "Run Analysis", the backend runs the sandbox. Upon receiving a successful `Report`, the state `activeBottomTab` is automatically updated to `"problems"` to focus the operator's attention on the results.
3. **Dynamic Plugin Orchestration:** Adding a plugin path in the Sidebar's plugin view registers it in the `plugins` state array. This dynamically adds it to the target selector inside the main scanner dropdown, making it instantly runnable.

---

## How to Extend the UI

### Adding a new Sidebar Panel
1. Add an icon to [ActivityBar.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/ActivityBar.tsx) with a specific state key (e.g. `setActiveView('history')`).
2. Update the condition check inside [SideBar.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/SideBar.tsx) to match your view and render custom explorer views:
   ```tsx
   if (activeView === 'history') {
     return <div>Historical scans...</div>;
   }
   ```

### Adding a new Settings Parameter
1. Register a new state in `App.tsx` (e.g., `const [reportsPath, setReportsPath] = useState("reports");`).
2. Pass the state and setter into [SettingsModal.tsx](file:///home/sohidul/developer_workspace/projects/aloevol/poly.glid_security_workspace/apps/desktop/src/components/layout/SettingsModal.tsx).
3. Create an input field in the "Engine" tab to edit the value.
