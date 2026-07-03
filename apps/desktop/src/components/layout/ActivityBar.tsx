import { FileSearch, Settings, Code2, Network } from 'lucide-react';

interface ActivityBarProps {
  activeView: string;
  setActiveView: (view: string) => void;
  onSettingsClick: () => void;
}

export function ActivityBar({ activeView, setActiveView, onSettingsClick }: ActivityBarProps) {
  return (
    <div className="w-12 bg-gray-900 border-r border-gray-800 flex flex-col items-center py-4 space-y-6 shrink-0">
      <div className="flex flex-col space-y-6 flex-1">
        <button 
          onClick={() => setActiveView('explorer')}
          className={`transition-colors group relative ${activeView === 'explorer' ? 'text-blue-500' : 'text-gray-400 hover:text-white'}`}
        >
          <FileSearch size={24} />
          <div className="absolute left-14 bg-gray-800 text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none transition-opacity z-50">Explorer</div>
        </button>
        <button 
          onClick={() => setActiveView('plugins')}
          className={`transition-colors group relative ${activeView === 'plugins' ? 'text-blue-500' : 'text-gray-400 hover:text-white'}`}
        >
          <Network size={24} />
          <div className="absolute left-14 bg-gray-800 text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none transition-opacity z-50">Plugins</div>
        </button>
      </div>
      <div className="pb-4">
        <button 
          onClick={onSettingsClick}
          className="text-gray-400 hover:text-white transition-colors group relative"
        >
          <Settings size={24} />
          <div className="absolute left-14 bg-gray-800 text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 whitespace-nowrap pointer-events-none transition-opacity z-50">Settings</div>
        </button>
      </div>
    </div>
  );
}
