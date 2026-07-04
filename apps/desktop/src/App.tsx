import { useState, useEffect } from "react";
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

  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [selectedPlugin, setSelectedPlugin] = useState("");

  const [report, setReport] = useState<Report | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const refreshPlugins = async () => {
    try {
      const list = await invoke<PluginInfo[]>("get_installed_plugins");
      setPlugins(list);
      if (list.length > 0 && !list.some(p => p.id === selectedPlugin)) {
        setSelectedPlugin(list[0].id);
      }
    } catch (e) {
      console.error("Failed to load installed plugins from registry:", e);
    }
  };

  useEffect(() => {
    refreshPlugins();
  }, []);

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

  const handleAddPlugin = async (name: string, path: string) => {
    try {
      setLoading(true);
      setError(null);
      const entry = await invoke<PluginInfo>("install_plugin", { srcPath: path });
      await refreshPlugins();
      setSelectedPlugin(entry.id);
    } catch (e: any) {
      console.error("Failed to install plugin:", e);
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleRemovePlugin = async (id: string) => {
    try {
      setLoading(true);
      setError(null);
      await invoke("uninstall_plugin", { pluginId: id });
      await refreshPlugins();
    } catch (e: any) {
      console.error("Failed to uninstall plugin:", e);
      setError(e.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleTogglePluginEnabled = async (id: string, enabled: boolean) => {
    try {
      await invoke("toggle_plugin_enabled", { pluginId: id, enabled });
      await refreshPlugins();
    } catch (e: any) {
      console.error("Failed to toggle plugin:", e);
      setError(e.toString());
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
      if (res.panel) {
        setActiveEditorTab("result");
      }
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
          onTogglePluginEnabled={handleTogglePluginEnabled}
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
            report={report}
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
