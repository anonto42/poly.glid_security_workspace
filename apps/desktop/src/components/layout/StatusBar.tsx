import { ShieldCheck, Cpu } from 'lucide-react';

export function StatusBar() {
  return (
    <div className="h-6 bg-blue-600 text-white flex items-center px-3 text-xs font-medium justify-between shrink-0">
      <div className="flex items-center space-x-4">
        <div className="flex items-center space-x-1 cursor-pointer hover:bg-white/20 px-2 py-0.5 rounded transition-colors">
          <ShieldCheck size={14} />
          <span>PolyGlid Core Ready</span>
        </div>
        <div className="flex items-center space-x-1 cursor-pointer hover:bg-white/20 px-2 py-0.5 rounded transition-colors">
          <Cpu size={14} />
          <span>Wasmtime Engine</span>
        </div>
      </div>
      <div className="flex items-center space-x-4">
        <span className="cursor-pointer hover:bg-white/20 px-2 py-0.5 rounded transition-colors">Fuel Limit: 25M</span>
        <span className="cursor-pointer hover:bg-white/20 px-2 py-0.5 rounded transition-colors">Plugins: 1</span>
      </div>
    </div>
  );
}
