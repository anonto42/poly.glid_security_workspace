import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Report } from "./types";
import { ActivityBar } from "./components/layout/ActivityBar";
import { SideBar } from "./components/layout/SideBar";
import { EditorArea } from "./components/layout/EditorArea";
import { BottomPanel } from "./components/layout/BottomPanel";
import { StatusBar } from "./components/layout/StatusBar";
import { SettingsModal } from "./components/layout/SettingsModal";
import { PluginInfo } from "./types";

function App() {
  const [activeView, setActiveView] = useState("explorer");
  const [activeEditorTab, setActiveEditorTab] = useState("dashboard");
  const [activeBottomTab, setActiveBottomTab] = useState("problems");
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  
  const [targets, setTargets] = useState(["example.com", "google.com", "github.com"]);
  const [selectedTarget, setSelectedTarget] = useState("example.com");
  const [fuelLimit, setFuelLimit] = useState(25000000);

  const [plugins, setPlugins] = useState<PluginInfo[]>([
    { name: "recon_probe.wasm", path: "../../../target/wasm32-wasip1/debug/recon_probe.component.wasm" }
  ]);
  const [selectedPlugin, setSelectedPlugin] = useState(plugins[0].path);

  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleTargetSelect = (tgt: string) => {
    setSelectedTarget(tgt);
    setActiveEditorTab("dashboard");
  };

  const handleAddTarget = (tgt: string) => {
    if (!targets.includes(tgt)) {
      setTargets([...targets, tgt]);
    }
  };

  const handleRemoveTarget = (tgt: string) => {
    const nextTargets = targets.filter(t => t !== tgt);
    setTargets(nextTargets);
    if (selectedTarget === tgt && nextTargets.length > 0) {
      setSelectedTarget(nextTargets[0]);
    }
  };

  const handleAddPlugin = (name: string, path: string) => {
    if (!plugins.some(p => p.path === path)) {
      const newPlugin = { name, path };
      setPlugins([...plugins, newPlugin]);
      setSelectedPlugin(path);
    }
  };

  const handleRemovePlugin = (path: string) => {
    const nextPlugins = plugins.filter(p => p.path !== path);
    setPlugins(nextPlugins);
    if (selectedPlugin === path && nextPlugins.length > 0) {
      setSelectedPlugin(nextPlugins[0].path);
    }
  };

  async function handleRunPlugin(target: string) {
    setLoading(true);
    setError(null);
    setReport(null);
    try {
      const res: Report = await invoke("run_plugin", { pluginPath: selectedPlugin, target });
      setReport(res);
      setActiveBottomTab("problems");
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#1e1e1e] text-gray-300 font-sans selection:bg-blue-500/30">
      <div className="flex flex-1 overflow-hidden">
        <ActivityBar 
          activeView={activeView}
          setActiveView={setActiveView}
          onSettingsClick={() => setIsSettingsOpen(true)}
        />
        <SideBar 
          activeView={activeView}
          targets={targets}
          selectedTarget={selectedTarget}
          onTargetSelect={handleTargetSelect}
          onAddTarget={handleAddTarget}
          onRemoveTarget={handleRemoveTarget}
          plugins={plugins}
          onAddPlugin={handleAddPlugin}
          onRemovePlugin={handleRemovePlugin}
          fuelLimit={fuelLimit}
          setFuelLimit={setFuelLimit}
        />
        
        <div className="flex flex-col flex-1 min-w-0">
          <EditorArea 
            activeTab={activeEditorTab}
            setActiveTab={setActiveEditorTab}
            target={selectedTarget}
            setTarget={setSelectedTarget}
            plugins={plugins}
            selectedPlugin={selectedPlugin}
            setSelectedPlugin={setSelectedPlugin}
            onRunPlugin={handleRunPlugin}
            loading={loading}
            error={error}
          />
          <BottomPanel 
            activeTab={activeBottomTab}
            setActiveTab={setActiveBottomTab}
            report={report} 
          />
        </div>
      </div>
      
      <StatusBar />
      
      <SettingsModal 
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        fuelLimit={fuelLimit}
        setFuelLimit={setFuelLimit}
      />
    </div>
  );
}

export default App;
