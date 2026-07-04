import { useState } from 'react';
import { Plus, Trash2, Shield, Globe, Award, Key, Hash } from 'lucide-react';
import { PluginInfo } from '../../types';

interface SideBarProps {
  activeView: string;
  targets: string[];
  selectedTarget: string;
  onTargetSelect: (target: string) => void;
  onAddTarget: (target: string) => void;
  onRemoveTarget: (target: string) => void;
  plugins: PluginInfo[];
  onAddPlugin: (name: string, path: string) => void;
  onRemovePlugin: (id: string) => void;
  onTogglePluginEnabled: (id: string, enabled: boolean) => void;
  fuelLimit: number;
  setFuelLimit: (limit: number) => void;
}

export function SideBar({
  activeView,
  targets,
  selectedTarget,
  onTargetSelect,
  onAddTarget,
  onRemoveTarget,
  plugins,
  onAddPlugin,
  onRemovePlugin,
  onTogglePluginEnabled,
  fuelLimit,
  setFuelLimit,
}: SideBarProps) {
  const [newTarget, setNewTarget] = useState('');
  
  const [newPluginName, setNewPluginName] = useState('');
  const [newPluginPath, setNewPluginPath] = useState('');

  const handleAddSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newTarget.trim()) {
      onAddTarget(newTarget.trim());
      setNewTarget('');
    }
  };

  const handleAddPluginSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (newPluginPath.trim()) {
      onAddPlugin(newPluginName.trim() || 'Custom Plugin', newPluginPath.trim());
      setNewPluginName('');
      setNewPluginPath('');
    }
  };

  if (activeView === 'plugins') {
    return (
      <div className="w-64 bg-[#151515] border-r border-gray-800 flex flex-col shrink-0 text-gray-300">
        <div className="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider border-b border-gray-800">
          PLUGINS MANAGEMENT
        </div>
        
        <div className="flex-1 overflow-y-auto p-3 space-y-4">
          <form onSubmit={handleAddPluginSubmit} className="bg-[#1e1e1e] p-3 rounded border border-gray-800 space-y-3">
            <div className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
              Install Workspace Plugin
            </div>
            <div className="space-y-2">
              <input
                type="text"
                placeholder="Path: /absolute/path/to/plugin.wasm"
                value={newPluginPath}
                onChange={(e) => setNewPluginPath(e.target.value)}
                className="w-full bg-[#151515] text-xs border border-gray-800 rounded px-2 py-1.5 focus:outline-none focus:border-blue-500 font-mono"
                required
              />
            </div>
            <button 
              type="submit" 
              className="w-full bg-blue-600 hover:bg-blue-700 text-white text-xs font-semibold py-1.5 rounded transition-colors cursor-pointer flex items-center justify-center space-x-1"
            >
              <Plus size={12} />
              <span>Install Plugin</span>
            </button>
          </form>

          <div className="space-y-3">
            <div className="text-[10px] font-semibold text-gray-500 uppercase tracking-wider">
              Installed Plugins
            </div>
            
            <div className="space-y-3">
              {plugins.map((plugin) => (
                <div key={plugin.id} className={`bg-[#1e1e1e] p-3 rounded border flex flex-col gap-2 group ${plugin.status === 'Enabled' ? 'border-gray-800' : 'border-gray-800/40 opacity-60'}`}>
                  <div className="flex justify-between items-start gap-1">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center space-x-1.5 text-green-400 font-medium text-xs">
                        <span className={`inline-block w-1.5 h-1.5 rounded-full shrink-0 ${plugin.status === 'Enabled' ? 'bg-green-500' : 'bg-red-500'}`}></span>
                        <span className="truncate">{plugin.name}</span>
                      </div>
                      <div className="text-[9px] text-gray-500 font-mono truncate mt-0.5" title={plugin.id}>
                        {plugin.id}
                      </div>
                    </div>
                    <div className="flex items-center space-x-1.5 shrink-0">
                      <input
                        type="checkbox"
                        checked={plugin.status === 'Enabled'}
                        onChange={(e) => onTogglePluginEnabled(plugin.id, e.target.checked)}
                        className="w-3.5 h-3.5 rounded bg-[#151515] border border-gray-800 cursor-pointer"
                        title={plugin.status === 'Enabled' ? "Disable plugin" : "Enable plugin"}
                      />
                      <button 
                        onClick={() => onRemovePlugin(plugin.id)}
                        className="text-gray-500 hover:text-red-400 p-0.5"
                        title="Uninstall plugin"
                      >
                        <Trash2 size={12} />
                      </button>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 text-[10px] text-gray-400">
                    <Award size={10} className="text-blue-400 shrink-0" />
                    <span>Version {plugin.version}</span>
                    {plugin.author && <span className="text-gray-600">by {plugin.author}</span>}
                  </div>

                  {plugin.description && (
                    <p className="text-[10px] text-gray-500 leading-normal border-t border-gray-800/40 pt-1.5">
                      {plugin.description}
                    </p>
                  )}

                  <div className="flex items-center gap-1 text-[8px] text-gray-500 font-mono truncate border-t border-gray-800/40 pt-1">
                    <Hash size={8} />
                    <span className="truncate" title={plugin.checksum}>{plugin.checksum.substring(0, 16)}...</span>
                  </div>

                  {plugin.capabilities && plugin.capabilities.length > 0 && (
                    <div className="border-t border-gray-800/40 pt-1.5">
                      <div className="text-[9px] font-semibold text-gray-500 uppercase tracking-wider mb-1 flex items-center gap-1">
                        <Key size={10} className="text-yellow-500" />
                        <span>Required Scopes</span>
                      </div>
                      <div className="flex flex-wrap gap-1">
                        {plugin.capabilities.map((cap) => (
                          <span key={cap} className="px-1.5 py-0.5 rounded bg-[#27273a] text-yellow-500 font-mono text-[9px] border border-yellow-500/10">
                            {cap}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (activeView === 'settings') {
    return (
      <div className="w-64 bg-[#151515] border-r border-gray-800 flex flex-col shrink-0">
        <div className="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider border-b border-gray-800">
          SYSTEM SETTINGS
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-6">
          <div className="space-y-2">
            <label className="text-xs font-semibold text-gray-400 uppercase block">Max WASM Fuel</label>
            <input 
              type="number"
              value={fuelLimit}
              onChange={(e) => setFuelLimit(Number(e.target.value))}
              className="w-full bg-[#1e1e1e] border border-gray-800 rounded px-2 py-1 text-sm text-white focus:outline-none focus:border-blue-500 font-mono"
            />
            <p className="text-[10px] text-gray-500">Limits execution cycles in Wasmtime sandbox to prevent infinite loops.</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="w-64 bg-[#151515] border-r border-gray-800 flex flex-col shrink-0">
      <div className="p-3 text-xs font-semibold text-gray-400 uppercase tracking-wider border-b border-gray-800">
        EXPLORER: TARGETS
      </div>
      
      <div className="flex-1 overflow-y-auto">
        <div className="p-2 space-y-2">
          <form onSubmit={handleAddSubmit} className="flex space-x-1 mb-2">
            <input
              type="text"
              placeholder="Add target..."
              value={newTarget}
              onChange={(e) => setNewTarget(e.target.value)}
              className="flex-1 bg-[#1e1e1e] text-xs border border-gray-800 rounded px-2 py-1 focus:outline-none focus:border-blue-500"
            />
            <button type="submit" className="p-1 bg-gray-800 hover:bg-gray-700 rounded text-gray-300">
              <Plus size={14} />
            </button>
          </form>

          <div className="space-y-1">
            {targets.map((tgt) => (
              <div 
                key={tgt}
                onClick={() => onTargetSelect(tgt)}
                className={`flex items-center justify-between px-2 py-1 text-sm rounded cursor-pointer group ${selectedTarget === tgt ? 'bg-gray-800 text-white border-l-2 border-blue-500 pl-1.5' : 'text-gray-400 hover:bg-gray-900/50 hover:text-white'}`}
              >
                <div className="flex items-center space-x-2 truncate">
                  <Globe size={14} className="text-gray-500 shrink-0" />
                  <span className="truncate">{tgt}</span>
                </div>
                <button 
                  onClick={(e) => {
                    e.stopPropagation();
                    onRemoveTarget(tgt);
                  }}
                  className="text-gray-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity p-0.5"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            ))}
          </div>
        </div>

        <div className="mt-4 border-t border-gray-800/50 p-2">
          <div className="text-[10px] font-semibold text-gray-500 uppercase tracking-wider px-2 mb-2">
            ACTIVE PLUGINS
          </div>
          <div className="space-y-2">
            {plugins.map((plugin) => (
              <div key={plugin.id} className={`px-2 py-1 rounded bg-[#1c1c1c]/50 border mx-2 flex flex-col gap-1 min-w-0 ${plugin.status === 'Enabled' ? 'border-gray-800/40' : 'border-red-900/20 opacity-50'}`}>
                <div className="text-xs text-green-400 font-mono flex items-center space-x-2 truncate">
                  <span className={`inline-block w-1.5 h-1.5 rounded-full shrink-0 ${plugin.status === 'Enabled' ? 'bg-green-500' : 'bg-red-500'}`}></span>
                  <span className="truncate">{plugin.name}</span>
                </div>
                {plugin.capabilities && plugin.capabilities.length > 0 && (
                  <div className="flex flex-wrap gap-1 pl-3.5 mt-0.5">
                    {plugin.capabilities.map((cap) => (
                      <span key={cap} className="px-1 text-[8px] rounded bg-[#27273a] text-yellow-500 font-mono">
                        {cap.split(' ')[0]}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
