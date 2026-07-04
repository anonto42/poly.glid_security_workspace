import React, { useState, useEffect, useRef } from 'react';

const API_BASE = window.location.port === '3000' ? 'http://127.0.0.1:8080' : window.location.origin;
const WS_BASE = API_BASE.replace('http://', 'ws://').replace('https://', 'wss://');

interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  status: string;
  capabilities: string[];
}

interface Execution {
  job_id: string;
  plugin_id: string;
  target: string;
  state: string;
  started_at: number;
  duration_ms: number;
  error_message: string | null;
  fuel_consumed: number;
}

interface Report {
  id: string;
  job_id: string;
  plugin_id: string;
  target: string;
  summary: string;
  issues: any[];
  filepath: string;
  created_at: number;
}

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [authToken, setAuthToken] = useState(() => localStorage.getItem('pg_auth_token') || '');
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [targets, setTargets] = useState<string[]>([]);
  const [executions, setExecutions] = useState<Execution[]>([]);
  const [reports, setReports] = useState<Report[]>([]);
  const [selectedReport, setSelectedReport] = useState<Report | null>(null);

  // Form states
  const [newTarget, setNewTarget] = useState('');
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);
  const [selectedTarget, setSelectedTarget] = useState('');
  const [configThreads, setConfigThreads] = useState('20');
  const [configTimeout, setConfigTimeout] = useState('5');
  const [configPorts, setConfigPorts] = useState('common');

  // Monitor states
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const [liveLogs, setLiveLogs] = useState<string[]>([]);
  const [liveStage, setLiveStage] = useState('Queued');
  const [liveFuel, setLiveFuel] = useState<number>(0);
  const [liveDuration, setLiveDuration] = useState<number>(0);

  // Toast notification
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'info' | 'error' | 'warning' } | null>(null);

  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    localStorage.setItem('pg_auth_token', authToken);
    if (authToken) {
      fetchData();
      connectWebSocket();
    }
  }, [authToken]);

  const showToast = (message: string, type: 'success' | 'info' | 'error' | 'warning' = 'info') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const getHeaders = () => {
    return {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`
    };
  };

  const fetchData = async () => {
    try {
      // Plugins
      const plRes = await fetch(`${API_BASE}/api/v1/plugins`, { headers: getHeaders() });
      if (plRes.ok) {
        const data = await plRes.json();
        setPlugins(data.map((p: any) => ({
          id: p.id.id,
          name: p.name,
          version: p.version,
          author: p.author,
          description: p.description,
          status: p.status,
          capabilities: p.capabilities
        })));
      }

      // Targets
      const tgRes = await fetch(`${API_BASE}/api/v1/targets`, { headers: getHeaders() });
      if (tgRes.ok) {
        const data = await tgRes.json();
        setTargets(data);
        if (data.length > 0) setSelectedTarget(data[0]);
      }

      // Executions
      const exRes = await fetch(`${API_BASE}/api/v1/executions`, { headers: getHeaders() });
      if (exRes.ok) {
        setExecutions(await exRes.json());
      }

      // Reports
      const rpRes = await fetch(`${API_BASE}/api/v1/reports`, { headers: getHeaders() });
      if (rpRes.ok) {
        setReports(await rpRes.json());
      }
    } catch (err) {
      showToast('Connection to server failed. Verify API token.', 'error');
    }
  };

  const connectWebSocket = () => {
    if (wsRef.current) wsRef.current.close();
    
    const ws = new WebSocket(`${WS_BASE}/ws/v1/events`);
    wsRef.current = ws;

    ws.onmessage = (event) => {
      try {
        const envelope = JSON.parse(event.data);
        const { type, payload } = envelope;

        if (type.includes('jobstatechanged')) {
          const { job_id, state } = payload;
          setExecutions(prev => prev.map(e => e.job_id === job_id ? { ...e, state } : e));
          
          if (job_id === activeJobId) {
            setLiveStage(state);
            if (state === 'Completed' || state === 'Failed') {
              fetchData();
              showToast(`Scan job ${state.toLowerCase()} successfully!`, state === 'Completed' ? 'success' : 'error');
            }
          }
        } else if (type.includes('joblog')) {
          const { job_id, message } = payload;
          if (job_id === activeJobId) {
            if (message.startsWith('[STAGE:')) {
              const stageName = message.substring(message.indexOf(':') + 1, message.indexOf(']')).trim();
              setLiveStage(stageName);
            }
            setLiveLogs(prev => [...prev, message]);
          }
        }
      } catch (e) {
        // ignore malformed ws frames
      }
    };

    ws.onclose = () => {
      setTimeout(connectWebSocket, 5000);
    };
  };

  const handleAddTarget = async () => {
    if (!newTarget.trim()) return;
    try {
      const res = await fetch(`${API_BASE}/api/v1/targets`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ name: newTarget.trim() })
      });
      if (res.ok) {
        showToast('Target registered successfully', 'success');
        setNewTarget('');
        fetchData();
      } else {
        showToast('Failed to register target', 'error');
      }
    } catch (err) {
      showToast('Error registering target', 'error');
    }
  };

  const handleRemoveTarget = async (name: string) => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/targets/${name}`, {
        method: 'DELETE',
        headers: getHeaders()
      });
      if (res.ok) {
        showToast('Target removed successfully', 'success');
        fetchData();
      }
    } catch (err) {
      showToast('Error removing target', 'error');
    }
  };

  const handleTogglePlugin = async (id: string, currentStatus: string) => {
    const nextStatus = currentStatus === 'Enabled' ? false : true;
    try {
      const res = await fetch(`${API_BASE}/api/v1/plugins/${id}/toggle`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ enabled: nextStatus })
      });
      if (res.ok) {
        showToast(`Plugin status ${nextStatus ? 'enabled' : 'disabled'}`, 'success');
        fetchData();
      }
    } catch (err) {
      showToast('Error toggling plugin', 'error');
    }
  };

  const handleLaunchScan = async () => {
    if (!selectedPlugin) {
      showToast('Select a plugin to execute scan', 'warning');
      return;
    }
    const target = selectedTarget || targets[0];
    if (!target) {
      showToast('Register at least one target', 'warning');
      return;
    }

    try {
      // 1. Configure settings
      await fetch(`${API_BASE}/api/v1/plugins/${selectedPlugin.id}/configure`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({
          threads: configThreads,
          timeout: configTimeout,
          ports: configPorts
        })
      });

      // 2. Launch execution
      const runRes = await fetch(`${API_BASE}/api/v1/executions`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({
          plugin_id: selectedPlugin.id,
          target
        })
      });

      if (runRes.ok) {
        const { job_id } = await runRes.json();
        setActiveJobId(job_id);
        setLiveLogs([`[SYSTEM] Job submitted. ID: ${job_id}`]);
        setLiveStage('Queued');
        setLiveFuel(0);
        setLiveDuration(0);
        setActiveTab('monitor');
        showToast('Scan initiated. Redirecting to live monitor...', 'success');
        fetchData();
      } else {
        showToast('Failed to trigger scan execution', 'error');
      }
    } catch (err) {
      showToast('Error launching scan', 'error');
    }
  };

  const handleDownload = (id: string, format: string) => {
    window.open(`${API_BASE}/api/v1/reports/${id}/download?format=${format}&Authorization=Bearer ${authToken}`);
  };

  return (
    <div style={{
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      backgroundColor: '#0d1117',
      color: '#c9d1d9',
      minHeight: '100vh',
      display: 'flex',
      flexDirection: 'column'
    }}>
      {/* Toast */}
      {toast && (
        <div style={{
          position: 'fixed',
          bottom: '24px',
          right: '24px',
          padding: '16px 24px',
          borderRadius: '8px',
          color: '#fff',
          fontWeight: 'bold',
          zIndex: 1000,
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
          backgroundColor: toast.type === 'success' ? '#238636' : (toast.type === 'error' ? '#da3637' : '#1f6feb'),
          transition: 'all 0.3s ease'
        }}>
          {toast.message}
        </div>
      )}

      {/* Header */}
      <header style={{
        background: 'linear-gradient(90deg, #1f2937, #111827)',
        padding: '16px 24px',
        borderBottom: '1px solid #30363d',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{
            background: 'linear-gradient(135deg, #58a6ff, #1f6feb)',
            width: '32px',
            height: '32px',
            borderRadius: '8px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontWeight: 'bold',
            color: '#fff'
          }}>PG</div>
          <span style={{ fontSize: '20px', fontWeight: 'bold', letterSpacing: '0.5px' }}>PolyGlid Control Panel</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <input
            type="password"
            placeholder="Admin API Token"
            value={authToken}
            onChange={(e) => setAuthToken(e.target.value)}
            style={{
              backgroundColor: '#161b22',
              border: '1px solid #30363d',
              borderRadius: '6px',
              padding: '8px 12px',
              color: '#c9d1d9',
              fontSize: '13px',
              width: '200px'
            }}
          />
          <button
            onClick={fetchData}
            style={{
              padding: '8px 16px',
              borderRadius: '6px',
              border: '1px solid #30363d',
              backgroundColor: '#21262d',
              color: '#c9d1d9',
              cursor: 'pointer'
            }}
          >
            Refresh
          </button>
        </div>
      </header>

      {/* Main Layout */}
      <div style={{ display: 'flex', flex: 1 }}>
        {/* Sidebar */}
        <aside style={{
          width: '240px',
          backgroundColor: '#161b22',
          borderRight: '1px solid #30363d',
          padding: '24px 12px',
          display: 'flex',
          flexDirection: 'column',
          gap: '8px'
        }}>
          {[
            { id: 'dashboard', label: 'Workspace Dashboard' },
            { id: 'plugins', label: 'Plugin Manager' },
            { id: 'targets', label: 'Target Manager' },
            { id: 'execution', label: 'Scan Launcher' },
            { id: 'monitor', label: 'Live Job Monitor' },
            { id: 'reports', label: 'Report Viewer' }
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              style={{
                textAlign: 'left',
                padding: '12px 16px',
                borderRadius: '6px',
                border: 'none',
                backgroundColor: activeTab === tab.id ? '#21262d' : 'transparent',
                color: activeTab === tab.id ? '#58a6ff' : '#8b949e',
                fontWeight: activeTab === tab.id ? '600' : 'normal',
                cursor: 'pointer',
                fontSize: '14px',
                transition: 'all 0.2s ease'
              }}
            >
              {tab.label}
            </button>
          ))}
        </aside>

        {/* Content Area */}
        <main style={{ flex: 1, padding: '32px', backgroundColor: '#0d1117' }}>
          
          {/* Dashboard */}
          {activeTab === 'dashboard' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0, marginBottom: '24px' }}>Workspace Status</h2>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '20px', marginBottom: '32px' }}>
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '24px', borderRadius: '8px' }}>
                  <div style={{ color: '#8b949e', fontSize: '14px' }}>Installed Plugins</div>
                  <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#fff', marginTop: '12px' }}>
                    {plugins.length} <span style={{ fontSize: '14px', color: '#8b949e' }}>({plugins.filter(p => p.status === 'Enabled').length} Active)</span>
                  </div>
                </div>
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '24px', borderRadius: '8px' }}>
                  <div style={{ color: '#8b949e', fontSize: '14px' }}>Scan Targets</div>
                  <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#fff', marginTop: '12px' }}>{targets.length}</div>
                </div>
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '24px', borderRadius: '8px' }}>
                  <div style={{ color: '#8b949e', fontSize: '14px' }}>Executions History</div>
                  <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#fff', marginTop: '12px' }}>
                    {executions.length} <span style={{ fontSize: '14px', color: '#da3637' }}>({executions.filter(e => e.state === 'Failed').length} Failed)</span>
                  </div>
                </div>
              </div>

              <h3 style={{ color: '#fff' }}>Recent Activity Logs</h3>
              <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', borderRadius: '8px', padding: '16px' }}>
                {executions.length === 0 ? (
                  <span style={{ color: '#8b949e' }}>No scan history recorded in database.</span>
                ) : (
                  executions.slice(0, 5).map(e => (
                    <div key={e.job_id} style={{ display: 'flex', justifyContent: 'space-between', padding: '12px 8px', borderBottom: '1px solid #21262d' }}>
                      <span>Plugin <code>{e.plugin_id}</code> completed execution on target <b>{e.target}</b></span>
                      <span style={{
                        color: e.state === 'Completed' ? '#238636' : '#da3637',
                        fontWeight: 'bold'
                      }}>{e.state}</span>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {/* Plugin Manager */}
          {activeTab === 'plugins' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Plugin Manager</h2>
              <p style={{ color: '#8b949e' }}>Toggle local WASM plugins or configure sandbox limits.</p>
              
              <div style={{ display: 'grid', gap: '16px', marginTop: '24px' }}>
                {plugins.map(p => (
                  <div key={p.id} style={{
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    padding: '24px',
                    borderRadius: '8px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center'
                  }}>
                    <div>
                      <h4 style={{ color: '#fff', margin: 0, fontSize: '18px' }}>{p.name} <span style={{ fontSize: '13px', color: '#8b949e' }}>v{p.version}</span></h4>
                      <p style={{ margin: '8px 0', fontSize: '14px', color: '#8b949e' }}>{p.description}</p>
                      <div style={{ display: 'flex', gap: '8px', fontSize: '12px', color: '#58a6ff' }}>
                        <span>Author: {p.author}</span> •
                        <span>Capabilities: {p.capabilities.join(', ') || 'none'}</span>
                      </div>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                      <button
                        onClick={() => handleTogglePlugin(p.id, p.status)}
                        style={{
                          padding: '8px 16px',
                          borderRadius: '6px',
                          border: 'none',
                          fontWeight: 'bold',
                          cursor: 'pointer',
                          backgroundColor: p.status === 'Enabled' ? '#da3637' : '#238636',
                          color: '#fff'
                        }}
                      >
                        {p.status === 'Enabled' ? 'Disable' : 'Enable'}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Target Manager */}
          {activeTab === 'targets' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Target Manager</h2>
              <div style={{ display: 'flex', gap: '12px', margin: '24px 0' }}>
                <input
                  type="text"
                  placeholder="Target domain, IP, or URL"
                  value={newTarget}
                  onChange={(e) => setNewTarget(e.target.value)}
                  style={{
                    flex: 1,
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    padding: '10px 16px',
                    color: '#c9d1d9',
                    fontSize: '14px'
                  }}
                />
                <button
                  onClick={handleAddTarget}
                  style={{
                    backgroundColor: '#238636',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '6px',
                    padding: '10px 24px',
                    fontWeight: 'bold',
                    cursor: 'pointer'
                  }}
                >
                  Add Target
                </button>
              </div>

              <ul style={{ listStyleType: 'none', padding: 0 }}>
                {targets.map(t => (
                  <li key={t} style={{
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    padding: '16px 20px',
                    borderRadius: '6px',
                    marginBottom: '10px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center'
                  }}>
                    <span>{t}</span>
                    <button
                      onClick={() => handleRemoveTarget(t)}
                      style={{
                        backgroundColor: 'transparent',
                        border: '1px solid #da3637',
                        color: '#da3637',
                        borderRadius: '4px',
                        padding: '4px 10px',
                        cursor: 'pointer',
                        fontSize: '12px'
                      }}
                    >
                      Delete
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Scan Launcher */}
          {activeTab === 'execution' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Launch Plugin Scan</h2>
              
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '24px', marginTop: '24px' }}>
                {/* Left Panel */}
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '24px', borderRadius: '8px' }}>
                  <label style={{ display: 'block', marginBottom: '8px', color: '#8b949e' }}>Select target</label>
                  <select
                    value={selectedTarget}
                    onChange={(e) => setSelectedTarget(e.target.value)}
                    style={{
                      width: '100%',
                      padding: '10px',
                      backgroundColor: '#0d1117',
                      border: '1px solid #30363d',
                      borderRadius: '6px',
                      color: '#c9d1d9',
                      marginBottom: '20px'
                    }}
                  >
                    {targets.map(t => <option key={t} value={t}>{t}</option>)}
                  </select>

                  <label style={{ display: 'block', marginBottom: '8px', color: '#8b949e' }}>Select Plugin</label>
                  <select
                    value={selectedPlugin?.id || ''}
                    onChange={(e) => {
                      const pl = plugins.find(p => p.id === e.target.value);
                      if (pl) setSelectedPlugin(pl);
                    }}
                    style={{
                      width: '100%',
                      padding: '10px',
                      backgroundColor: '#0d1117',
                      border: '1px solid #30363d',
                      borderRadius: '6px',
                      color: '#c9d1d9',
                      marginBottom: '20px'
                    }}
                  >
                    <option value="">-- Choose Plugin --</option>
                    {plugins.filter(p => p.status === 'Enabled').map(p => (
                      <option key={p.id} value={p.id}>{p.name}</option>
                    ))}
                  </select>

                  <button
                    onClick={handleLaunchScan}
                    style={{
                      width: '100%',
                      padding: '12px',
                      backgroundColor: '#1f6feb',
                      color: '#fff',
                      border: 'none',
                      borderRadius: '6px',
                      fontWeight: 'bold',
                      cursor: 'pointer',
                      fontSize: '15px'
                    }}
                  >
                    Execute Scan Pipeline
                  </button>
                </div>

                {/* Right Panel: Dynamic settings form */}
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '24px', borderRadius: '8px' }}>
                  <h4 style={{ color: '#fff', margin: '0 0 16px 0' }}>Plugin Settings (Dynamic)</h4>
                  {selectedPlugin ? (
                    <div>
                      <div style={{ marginBottom: '16px' }}>
                        <label style={{ display: 'block', marginBottom: '6px', color: '#8b949e', fontSize: '13px' }}>Threads limit</label>
                        <input
                          type="number"
                          value={configThreads}
                          onChange={(e) => setConfigThreads(e.target.value)}
                          style={{
                            width: '100%',
                            backgroundColor: '#0d1117',
                            border: '1px solid #30363d',
                            padding: '8px',
                            color: '#c9d1d9',
                            borderRadius: '6px'
                          }}
                        />
                      </div>
                      <div style={{ marginBottom: '16px' }}>
                        <label style={{ display: 'block', marginBottom: '6px', color: '#8b949e', fontSize: '13px' }}>Timeout (seconds)</label>
                        <input
                          type="number"
                          value={configTimeout}
                          onChange={(e) => setConfigTimeout(e.target.value)}
                          style={{
                            width: '100%',
                            backgroundColor: '#0d1117',
                            border: '1px solid #30363d',
                            padding: '8px',
                            color: '#c9d1d9',
                            borderRadius: '6px'
                          }}
                        />
                      </div>
                      <div>
                        <label style={{ display: 'block', marginBottom: '6px', color: '#8b949e', fontSize: '13px' }}>Port configuration</label>
                        <select
                          value={configPorts}
                          onChange={(e) => setConfigPorts(e.target.value)}
                          style={{
                            width: '100%',
                            backgroundColor: '#0d1117',
                            border: '1px solid #30363d',
                            padding: '8px',
                            color: '#c9d1d9',
                            borderRadius: '6px'
                          }}
                        >
                          <option value="common">Common Ports Only</option>
                          <option value="full">Full 65535 Scan</option>
                        </select>
                      </div>
                    </div>
                  ) : (
                    <span style={{ color: '#8b949e', fontSize: '14px' }}>Select a plugin on the left to configure settings.</span>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Live Monitor */}
          {activeTab === 'monitor' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Live Job Monitor</h2>
              {activeJobId ? (
                <div style={{ marginTop: '24px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '16px' }}>
                    <div>
                      <span>Job ID: <code>{activeJobId}</code></span>
                      <div style={{ marginTop: '8px' }}>Stage: <b style={{ color: '#58a6ff' }}>{liveStage}</b></div>
                    </div>
                    <span style={{
                      backgroundColor: liveStage === 'Completed' ? '#238636' : '#1f6feb',
                      padding: '6px 12px',
                      borderRadius: '4px',
                      color: '#fff',
                      fontWeight: 'bold',
                      alignSelf: 'flex-start'
                    }}>{liveStage}</span>
                  </div>

                  <div style={{
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    padding: '20px',
                    borderRadius: '8px',
                    fontFamily: 'monospace',
                    fontSize: '13px',
                    maxHeight: '400px',
                    overflowY: 'auto',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: '4px'
                  }}>
                    {liveLogs.map((log, i) => <div key={i} style={{ color: log.includes('error') ? '#da3637' : '#c9d1d9' }}>{log}</div>)}
                  </div>
                </div>
              ) : (
                <span style={{ color: '#8b949e', display: 'block', marginTop: '24px' }}>No active running scan pipeline log to monitor.</span>
              )}
            </div>
          )}

          {/* Report Viewer */}
          {activeTab === 'reports' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Report Viewer</h2>
              
              {!selectedReport ? (
                <div style={{ display: 'grid', gap: '12px', marginTop: '24px' }}>
                  {reports.map(r => (
                    <div key={r.id} style={{
                      backgroundColor: '#161b22',
                      border: '1px solid #30363d',
                      padding: '20px',
                      borderRadius: '8px',
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center'
                    }}>
                      <div>
                        <h4 style={{ color: '#fff', margin: 0 }}>Report for {r.target}</h4>
                        <span style={{ fontSize: '12px', color: '#8b949e' }}>Executed by {r.plugin_id} • Findings: {r.issues.length}</span>
                      </div>
                      <div style={{ display: 'flex', gap: '8px' }}>
                        <button
                          onClick={() => setSelectedReport(r)}
                          style={{
                            padding: '6px 12px',
                            backgroundColor: '#1f6feb',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer'
                          }}
                        >
                          View Details
                        </button>
                        <button
                          onClick={() => handleDownload(r.id, 'sarif')}
                          style={{
                            padding: '6px 12px',
                            backgroundColor: '#21262d',
                            color: '#c9d1d9',
                            border: '1px solid #30363d',
                            borderRadius: '4px',
                            cursor: 'pointer'
                          }}
                        >
                          SARIF
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div style={{ marginTop: '24px' }}>
                  <button
                    onClick={() => setSelectedReport(null)}
                    style={{
                      padding: '8px 16px',
                      backgroundColor: '#21262d',
                      color: '#c9d1d9',
                      border: '1px solid #30363d',
                      borderRadius: '6px',
                      cursor: 'pointer',
                      marginBottom: '20px'
                    }}
                  >
                    ← Back to reports
                  </button>

                  <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '24px', borderRadius: '8px' }}>
                    <h3 style={{ color: '#fff', margin: 0 }}>Summary Details</h3>
                    <p style={{ margin: '12px 0 24px 0', fontSize: '15px' }}>{selectedReport.summary}</p>

                    <h4 style={{ color: '#fff', borderBottom: '1px solid #30363d', paddingBottom: '8px' }}>Findings List ({selectedReport.issues.length})</h4>
                    {selectedReport.issues.map((issue, idx) => (
                      <div key={idx} style={{
                        padding: '16px',
                        border: '1px solid #30363d',
                        borderRadius: '6px',
                        marginBottom: '12px',
                        backgroundColor: '#0d1117'
                      }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                          <span style={{ fontWeight: 'bold', color: '#fff' }}>{issue.title}</span>
                          <span style={{
                            padding: '2px 8px',
                            borderRadius: '4px',
                            fontSize: '11px',
                            backgroundColor: '#da3637',
                            color: '#fff'
                          }}>{issue.severity}</span>
                        </div>
                        <p style={{ margin: '8px 0', fontSize: '14px' }}>{issue.description}</p>
                        <div style={{ fontSize: '13px', color: '#58a6ff', marginTop: '8px' }}>
                          <b>Recommendation:</b> {issue.recommendation}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

        </main>
      </div>
    </div>
  );
}

export default App;
