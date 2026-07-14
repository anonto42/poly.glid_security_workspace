import { Report } from '../../types';
import { AlertTriangle, Info, ShieldAlert, Terminal as TerminalIcon } from 'lucide-react';

interface BottomPanelProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  report: Report | null;
}

export function BottomPanel({ activeTab, setActiveTab, report }: BottomPanelProps) {
  const getIcon = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'info': return <Info size={16} className="text-blue-400" />;
      case 'low': return <AlertTriangle size={16} className="text-yellow-500" />;
      case 'medium': return <AlertTriangle size={16} className="text-orange-500" />;
      case 'high': return <ShieldAlert size={16} className="text-red-500" />;
      case 'critical': return <ShieldAlert size={16} className="text-purple-500" />;
      default: return <Info size={16} className="text-gray-400" />;
    }
  };

  const renderContent = () => {
    if (activeTab === 'output') {
      return (
        <div className="font-mono text-xs text-gray-400 space-y-1 p-2 select-text">
          <div>[info] core engine initialized with Wasmtime-WASI preview 1</div>
          <div>[info] loading configuration from AppConfig::development()</div>
          <div>[info] reports directory mapped to "reports"</div>
          <div>[info] capability grants: dns-resolve=* (auto), report-write=* (auto)</div>
          {report && (
            <>
              <div className="text-blue-400">[info] executing plugin {report.plugin_name} on {report.target_tested}</div>
              <div className="text-green-500">[success] sandbox execute completed successfully in 0.15s</div>
            </>
          )}
        </div>
      );
    }

    if (activeTab === 'terminal') {
      return (
        <div className="font-mono text-xs text-green-400 p-2 flex flex-col h-full select-text">
          <div className="flex items-center space-x-2 text-gray-500 mb-2">
            <TerminalIcon size={12} />
            <span>Interactive Host Shell (polyglid-cli emulation)</span>
          </div>
          <div>polyglid --version</div>
          <div className="text-gray-400">polyglid v0.1.0</div>
          <div className="mt-2 text-gray-500">❯ _</div>
        </div>
      );
    }

    if (!report) {
      return (
        <div className="flex-1 flex items-center justify-center text-gray-500 text-sm p-4">
          No active scans. Choose a target and run the analysis.
        </div>
      );
    }

    return (
      <div className="p-2 space-y-4">
        {report.issues.length === 0 ? (
          <div className="text-green-400 text-sm font-mono p-2">
            {report.summary} - No security observations.
          </div>
        ) : (
          <div className="space-y-3">
            <div className="text-gray-400 text-xs uppercase tracking-wider font-semibold px-2 mb-2">
              Issues reported from {report.plugin_name}
            </div>
            {report.issues.map((issue, idx) => (
              <div key={idx} className="flex space-x-3 items-start hover:bg-[#2a2d2e] p-2 rounded transition-colors select-text">
                <div className="mt-0.5 shrink-0">
                  {getIcon(issue.severity)}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-baseline space-x-3">
                    <h4 className="text-sm font-medium text-gray-200">{issue.title}</h4>
                    <span className="text-xs text-gray-500 font-mono">[{issue.severity.toUpperCase()}]</span>
                  </div>
                  <p className="text-gray-400 text-sm mt-1">{issue.description}</p>
                  <p className="text-blue-400 text-sm mt-1 font-mono">{issue.recommendation}</p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="h-72 bg-[#1e1e1e] border-t border-gray-800 flex flex-col">
      <div className="flex bg-[#1e1e1e] border-b border-gray-800 px-4 items-center select-none shrink-0">
        <div 
          onClick={() => setActiveTab('problems')}
          className={`px-4 py-2 text-sm uppercase tracking-wider font-semibold cursor-pointer flex items-center space-x-2 border-b-2 ${activeTab === 'problems' ? 'text-white border-blue-500' : 'text-gray-500 border-transparent hover:text-gray-300'}`}
        >
          <span>Problems</span>
          {report && <span className="bg-gray-750 text-gray-400 rounded-full px-2 py-0.5 text-xs">{report.issues.length}</span>}
        </div>
        <div 
          onClick={() => setActiveTab('output')}
          className={`px-4 py-2 text-sm uppercase tracking-wider font-semibold cursor-pointer border-b-2 ${activeTab === 'output' ? 'text-white border-blue-500' : 'text-gray-500 border-transparent hover:text-gray-300'}`}
        >
          Output
        </div>
        <div 
          onClick={() => setActiveTab('terminal')}
          className={`px-4 py-2 text-sm uppercase tracking-wider font-semibold cursor-pointer border-b-2 ${activeTab === 'terminal' ? 'text-white border-blue-500' : 'text-gray-500 border-transparent hover:text-gray-300'}`}
        >
          Terminal
        </div>
      </div>
      
      <div className="flex-1 overflow-y-auto p-2 bg-[#1e1e1e]">
        {renderContent()}
      </div>
    </div>
  );
}
