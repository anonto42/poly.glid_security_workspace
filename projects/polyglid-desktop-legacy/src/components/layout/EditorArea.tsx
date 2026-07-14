import { Play, AlertCircle } from 'lucide-react';
import { PluginInfo, Report } from '../../types';
import { PanelRenderer } from './PanelRenderer';

interface EditorAreaProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  target: string;
  setTarget: (target: string) => void;
  plugins: PluginInfo[];
  selectedPlugin: string;
  setSelectedPlugin: (path: string) => void;
  onRunPlugin: (target: string) => Promise<void>;
  loading: boolean;
  error: string | null;
  report: Report | null;
}

export function EditorArea({
  activeTab,
  setActiveTab,
  target,
  setTarget,
  plugins,
  selectedPlugin,
  setSelectedPlugin,
  onRunPlugin,
  loading,
  error,
  report,
}: EditorAreaProps) {
  const pluginRustCode = `//! Harmless first-party demo plugin logic.

wit_bindgen::generate!({
    world: "security-tool",
    path: "../polyglid-contracts",
});

use crate::polyglid::engine::{
    dns, reports,
    types::{Issue, Severity},
};

struct ReconProbe;

impl Guest for ReconProbe {
    fn execute(target: String) -> Result<PluginReport, String> {
        let mut observations = analyze_target(&target, resolve_target(&target));
        // ... execute analysis logic ...
        Ok(PluginReport {
            plugin_name: "PolyGlid Recon Probe".to_string(),
            target_tested: target,
            issues,
            summary,
        })
    }
}`;

  return (
    <div className="flex-1 bg-[#1e1e1e] flex flex-col min-w-0">
      <div className="flex bg-[#2d2d2d] overflow-x-auto select-none border-b border-gray-800">
        <div 
          onClick={() => setActiveTab('dashboard')}
          className={`px-4 py-2 text-sm cursor-pointer flex items-center space-x-2 border-r border-gray-800 ${activeTab === 'dashboard' ? 'bg-[#1e1e1e] text-white border-t-2 border-blue-500' : 'text-gray-400 hover:bg-[#1e1e1e]/50'}`}
        >
          <span className="text-blue-400">⚡</span>
          <span>Scanner Configuration</span>
        </div>
        {report && (
          <div 
            onClick={() => setActiveTab('result')}
            className={`px-4 py-2 text-sm cursor-pointer flex items-center space-x-2 border-r border-gray-800 ${activeTab === 'result' ? 'bg-[#1e1e1e] text-white border-t-2 border-blue-500' : 'text-gray-400 hover:bg-[#1e1e1e]/50'}`}
          >
            <span className="text-green-400">📊</span>
            <span>Result Dashboard</span>
          </div>
        )}
        <div 
          onClick={() => setActiveTab('source')}
          className={`px-4 py-2 text-sm cursor-pointer flex items-center space-x-2 border-r border-gray-800 ${activeTab === 'source' ? 'bg-[#1e1e1e] text-white border-t-2 border-blue-500' : 'text-gray-400 hover:bg-[#1e1e1e]/50'}`}
        >
          <span className="text-orange-400">🦀</span>
          <span>recon_probe.rs</span>
        </div>
      </div>
      
      <div className="flex-1 overflow-y-auto">
        {activeTab === 'source' && (
          <div className="p-4 font-mono text-xs text-gray-300 leading-relaxed overflow-x-auto">
            <div className="text-gray-500 select-none border-b border-gray-800 pb-2 mb-2">// Read-only plugin source code</div>
            <pre className="text-green-400/90">{pluginRustCode}</pre>
          </div>
        )}
        {activeTab === 'result' && report && (
          <PanelRenderer layout={report.panel} />
        )}
        {activeTab === 'dashboard' && (
          <div className="p-8 max-w-3xl mx-auto space-y-8">
            <div>
              <h1 className="text-3xl font-light text-white mb-2 text-center">Security Scanner</h1>
              <p className="text-gray-400 text-center">Configure your target and launch the sandboxed plugin.</p>
            </div>

            <form 
              className="space-y-4"
              onSubmit={(e) => {
                e.preventDefault();
                onRunPlugin(target);
              }}
            >
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Target Domain / IP</label>
                <input
                  type="text"
                  className="w-full bg-[#3c3c3c] border border-gray-700 rounded p-2 text-white focus:outline-none focus:border-blue-500 font-mono text-sm"
                  value={target}
                  onChange={(e) => setTarget(e.target.value)}
                  placeholder="e.g. example.com"
                />
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-1">Selected Plugin</label>
                <select 
                  value={selectedPlugin}
                  onChange={(e) => setSelectedPlugin(e.target.value)}
                  className="w-full bg-[#3c3c3c] border border-gray-700 rounded p-2 text-white focus:outline-none focus:border-blue-500 text-sm"
                >
                  {plugins.map((plugin) => (
                    <option key={plugin.id} value={plugin.id}>
                      {plugin.name} ({plugin.id})
                    </option>
                  ))}
                </select>
                {plugins.find(p => p.id === selectedPlugin)?.status === 'Disabled' && (
                  <p className="text-xs text-red-400 mt-1">
                    This plugin is currently disabled in the workspace. Enable it in the Plugins side panel before running.
                  </p>
                )}
              </div>

              <button
                type="submit"
                disabled={loading || plugins.find(p => p.id === selectedPlugin)?.status === 'Disabled'}
                className="flex items-center justify-center space-x-2 w-full bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white p-2.5 rounded transition-colors mt-4 font-semibold text-sm cursor-pointer"
              >
                <Play size={16} />
                <span>{loading ? 'Executing in Sandbox...' : 'Run Analysis'}</span>
              </button>
            </form>

            {error && (
              <div className="flex items-start space-x-3 bg-red-900/30 border border-red-800 p-4 rounded text-red-400 mt-6">
                <AlertCircle className="shrink-0 mt-0.5" size={20} />
                <div>
                  <strong className="block mb-1 text-red-300">Execution Error</strong>
                  <span className="font-mono text-sm">{error}</span>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
