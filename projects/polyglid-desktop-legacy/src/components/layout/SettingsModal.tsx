import { useState } from 'react';
import { X, ShieldAlert, Cpu, Settings } from 'lucide-react';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  fuelLimit: number;
  setFuelLimit: (limit: number) => void;
}

export function SettingsModal({ isOpen, onClose, fuelLimit, setFuelLimit }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState('overview');

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm">
      <div className="bg-[#1e1e1e] border border-gray-800 rounded-lg w-[800px] h-[500px] shadow-2xl flex flex-col overflow-hidden text-gray-300">
        
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 bg-[#252526] border-b border-gray-800 shrink-0">
          <div className="flex items-center space-x-2">
            <Settings size={18} className="text-blue-400" />
            <span className="font-semibold text-sm text-white">PolyGlid Settings</span>
          </div>
          <button 
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors cursor-pointer"
          >
            <X size={18} />
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 flex overflow-hidden">
          
          {/* Modal Sidebar */}
          <div className="w-48 bg-[#252526] border-r border-gray-800 flex flex-col py-3 select-none shrink-0">
            <button 
              onClick={() => setActiveTab('overview')}
              className={`px-4 py-2 text-left text-sm transition-colors ${activeTab === 'overview' ? 'bg-[#1e1e1e] text-white font-medium border-l-2 border-blue-500' : 'text-gray-400 hover:text-white'}`}
            >
              Overview
            </button>
            <button 
              onClick={() => setActiveTab('engine')}
              className={`px-4 py-2 text-left text-sm transition-colors ${activeTab === 'engine' ? 'bg-[#1e1e1e] text-white font-medium border-l-2 border-blue-500' : 'text-gray-400 hover:text-white'}`}
            >
              Engine
            </button>
            <button 
              onClick={() => setActiveTab('plugins')}
              className={`px-4 py-2 text-left text-sm transition-colors ${activeTab === 'plugins' ? 'bg-[#1e1e1e] text-white font-medium border-l-2 border-blue-500' : 'text-gray-400 hover:text-white'}`}
            >
              Plugins
            </button>
          </div>

          {/* Modal Content */}
          <div className="flex-1 p-6 overflow-y-auto bg-[#1e1e1e]">
            
            {activeTab === 'overview' && (
              <div className="space-y-6">
                <div>
                  <h2 className="text-lg font-semibold text-white mb-1">System Overview</h2>
                  <p className="text-xs text-gray-400">Status of the sandboxed agent environment and system configuration.</p>
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div className="bg-[#252526] border border-gray-800 p-3 rounded space-y-1">
                    <div className="text-xs text-gray-500 uppercase tracking-wider font-semibold">Engine Runtime</div>
                    <div className="text-sm font-medium text-white flex items-center space-x-1.5">
                      <Cpu size={14} className="text-blue-400" />
                      <span>Wasmtime 46.0.1</span>
                    </div>
                  </div>
                  
                  <div className="bg-[#252526] border border-gray-800 p-3 rounded space-y-1">
                    <div className="text-xs text-gray-500 uppercase tracking-wider font-semibold">Sandboxing Model</div>
                    <div className="text-sm font-medium text-white flex items-center space-x-1.5">
                      <ShieldAlert size={14} className="text-green-400" />
                      <span>WASI Preview 1</span>
                    </div>
                  </div>
                </div>

                <div className="space-y-3">
                  <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Active Capabilities</h3>
                  <div className="bg-[#252526] border border-gray-800 p-3 rounded space-y-2 text-xs">
                    <div className="flex justify-between items-center text-gray-300">
                      <span className="font-mono">dns-resolve</span>
                      <span className="text-green-500">Auto-approved</span>
                    </div>
                    <div className="flex justify-between items-center text-gray-300">
                      <span className="font-mono">report-write</span>
                      <span className="text-green-500">Auto-approved</span>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'engine' && (
              <div className="space-y-6">
                <div>
                  <h2 className="text-lg font-semibold text-white mb-1">WASM Engine Configuration</h2>
                  <p className="text-xs text-gray-400">Configure safety thresholds and CPU fuel limits.</p>
                </div>

                <div className="space-y-4">
                  <div className="space-y-2">
                    <label className="text-xs font-semibold text-gray-400 uppercase block">Max WASM Fuel</label>
                    <input 
                      type="number"
                      value={fuelLimit}
                      onChange={(e) => setFuelLimit(Number(e.target.value))}
                      className="w-full bg-[#3c3c3c] border border-gray-800 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500 font-mono"
                    />
                    <p className="text-[10px] text-gray-500">Limits execution cycles in Wasmtime sandbox to prevent CPU starvation and infinite loops.</p>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'plugins' && (
              <div className="space-y-6">
                <div>
                  <h2 className="text-lg font-semibold text-white mb-1">Loaded Plugins</h2>
                  <p className="text-xs text-gray-400">Manage rules and configurations of active WebAssembly components.</p>
                </div>

                <div className="bg-[#252526] border border-gray-800 p-4 rounded flex justify-between items-center">
                  <div>
                    <div className="text-sm font-medium text-white">recon_probe.wasm</div>
                    <div className="text-xs text-gray-500 font-mono mt-0.5">plugins/recon_probe/src/lib.rs</div>
                  </div>
                  <span className="px-2 py-0.5 text-[10px] font-semibold text-green-400 border border-green-800 bg-green-950/20 rounded">Active</span>
                </div>
              </div>
            )}

          </div>

        </div>

        {/* Footer */}
        <div className="flex justify-end items-center px-4 py-3 bg-[#252526] border-t border-gray-800 shrink-0">
          <button 
            onClick={onClose}
            className="bg-blue-600 hover:bg-blue-700 text-white font-semibold py-1.5 px-4 rounded text-xs transition-colors cursor-pointer"
          >
            Done
          </button>
        </div>

      </div>
    </div>
  );
}
