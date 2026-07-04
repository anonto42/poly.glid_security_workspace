import React, { useState, useEffect } from 'react';

function App() {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [authToken, setAuthToken] = useState('');
  const [plugins, setPlugins] = useState<any[]>([]);
  const [targets, setTargets] = useState<string[]>([]);
  const [executions, setExecutions] = useState<any[]>([]);
  const [newTarget, setNewTarget] = useState('');
  const [notification, setNotification] = useState('');

  // Auto-connect server status mock data for demonstration
  useEffect(() => {
    // Seed mock data initially
    setPlugins([
      { id: 'polyglid.recon_probe', name: 'Recon Probe', version: '0.1.0', author: 'PolyGlid Team', status: 'Enabled' },
      { id: 'polyglid.vuln_scanner', name: 'Vulnerability Scanner', version: '0.2.0', author: 'Community', status: 'Disabled' }
    ]);
    setTargets(['localhost', 'example.com', 'test-range-1']);
    setExecutions([
      { job_id: 'job-101', plugin_id: 'polyglid.recon_probe', target: 'example.com', state: 'Completed', duration_ms: 120 },
      { job_id: 'job-102', plugin_id: 'polyglid.recon_probe', target: 'localhost', state: 'Failed', duration_ms: 45 }
    ]);
  }, []);

  const triggerNotification = (msg: string) => {
    setNotification(msg);
    setTimeout(() => setNotification(''), 4000);
  };

  const addTarget = () => {
    if (!newTarget.trim()) return;
    setTargets([...targets, newTarget.trim()]);
    triggerNotification(`Successfully registered target: ${newTarget}`);
    setNewTarget('');
  };

  const startScan = (pluginId: string) => {
    triggerNotification(`Scan scheduled for ${pluginId}. Execution state queued...`);
    const newJob = {
      job_id: `job-${Math.floor(Math.random() * 1000)}`,
      plugin_id: pluginId,
      target: targets[0] || 'example.com',
      state: 'Queued',
      duration_ms: 0
    };
    setExecutions([newJob, ...executions]);
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
      {/* Header bar */}
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
              padding: '6px 12px',
              color: '#c9d1d9',
              fontSize: '13px'
            }}
          />
          <span style={{
            fontSize: '12px',
            padding: '4px 8px',
            borderRadius: '4px',
            backgroundColor: authToken ? '#238636' : '#21262d',
            color: '#fff'
          }}>
            {authToken ? 'Authenticated' : 'Read Only'}
          </span>
        </div>
      </header>

      {notification && (
        <div style={{
          backgroundColor: '#1f6feb',
          color: '#fff',
          padding: '12px 24px',
          textAlign: 'center',
          fontWeight: '500',
          fontSize: '14px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)'
        }}>
          {notification}
        </div>
      )}

      {/* Main workspace */}
      <div style={{ display: 'flex', flex: 1 }}>
        {/* Sidebar Nav */}
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
            { id: 'dashboard', label: 'Dashboard' },
            { id: 'plugins', label: 'Plugins Registry' },
            { id: 'targets', label: 'Scan Targets' },
            { id: 'history', label: 'Execution History' },
            { id: 'reports', label: 'Reports Download' }
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              style={{
                textAlign: 'left',
                padding: '10px 16px',
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

        {/* Content Pane */}
        <main style={{ flex: 1, padding: '32px' }}>
          {activeTab === 'dashboard' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>System Metrics</h2>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '20px', marginBottom: '32px' }}>
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '20px', borderRadius: '8px' }}>
                  <div style={{ color: '#8b949e', fontSize: '14px' }}>Active Plugins</div>
                  <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#58a6ff', marginTop: '8px' }}>{plugins.filter(p => p.status === 'Enabled').length}</div>
                </div>
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '20px', borderRadius: '8px' }}>
                  <div style={{ color: '#8b949e', fontSize: '14px' }}>Registered Targets</div>
                  <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#58a6ff', marginTop: '8px' }}>{targets.length}</div>
                </div>
                <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', padding: '20px', borderRadius: '8px' }}>
                  <div style={{ color: '#8b949e', fontSize: '14px' }}>Total Jobs Executed</div>
                  <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#58a6ff', marginTop: '8px' }}>{executions.length}</div>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'plugins' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Plugins Registry</h2>
              <table style={{ width: '100%', borderCollapse: 'collapse', marginTop: '16px' }}>
                <thead>
                  <tr style={{ borderBottom: '2px solid #30363d', textAlign: 'left', color: '#8b949e' }}>
                    <th style={{ padding: '12px' }}>ID</th>
                    <th style={{ padding: '12px' }}>Name</th>
                    <th style={{ padding: '12px' }}>Version</th>
                    <th style={{ padding: '12px' }}>Status</th>
                    <th style={{ padding: '12px' }}>Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {plugins.map(p => (
                    <tr key={p.id} style={{ borderBottom: '1px solid #21262d' }}>
                      <td style={{ padding: '12px' }}><code>{p.id}</code></td>
                      <td style={{ padding: '12px', fontWeight: 'bold', color: '#fff' }}>{p.name}</td>
                      <td style={{ padding: '12px' }}>{p.version}</td>
                      <td style={{ padding: '12px' }}>
                        <span style={{
                          padding: '2px 6px',
                          borderRadius: '4px',
                          fontSize: '11px',
                          backgroundColor: p.status === 'Enabled' ? '#238636' : '#da3637',
                          color: '#fff'
                        }}>{p.status}</span>
                      </td>
                      <td style={{ padding: '12px' }}>
                        <button
                          onClick={() => startScan(p.id)}
                          disabled={p.status === 'Disabled'}
                          style={{
                            padding: '6px 12px',
                            borderRadius: '4px',
                            backgroundColor: p.status === 'Enabled' ? '#1f6feb' : '#21262d',
                            color: '#fff',
                            border: 'none',
                            cursor: p.status === 'Enabled' ? 'pointer' : 'not-allowed',
                            fontSize: '13px'
                          }}
                        >
                          Launch Scan
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {activeTab === 'targets' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Register Scan Targets</h2>
              <div style={{ display: 'flex', gap: '12px', margin: '24px 0' }}>
                <input
                  type="text"
                  placeholder="Target Host/IP (e.g. example.com)"
                  value={newTarget}
                  onChange={(e) => setNewTarget(e.target.value)}
                  style={{
                    flex: 1,
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    padding: '8px 16px',
                    color: '#c9d1d9',
                    fontSize: '14px'
                  }}
                />
                <button
                  onClick={addTarget}
                  style={{
                    backgroundColor: '#238636',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '6px',
                    padding: '8px 24px',
                    fontWeight: '600',
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
                    padding: '12px 20px',
                    borderRadius: '6px',
                    marginBottom: '8px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center'
                  }}>
                    <span>{t}</span>
                    <span style={{ color: '#8b949e', fontSize: '12px' }}>Verified</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {activeTab === 'history' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Asynchronous Job History</h2>
              <ul style={{ listStyleType: 'none', padding: 0, marginTop: '20px' }}>
                {executions.map(job => (
                  <li key={job.job_id} style={{
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    padding: '16px 20px',
                    borderRadius: '6px',
                    marginBottom: '12px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center'
                  }}>
                    <div>
                      <div style={{ fontWeight: 'bold', color: '#fff' }}>{job.plugin_id}</div>
                      <div style={{ color: '#8b949e', fontSize: '13px', marginTop: '4px' }}>Target: {job.target} • ID: {job.job_id}</div>
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                      <span style={{ fontSize: '13px' }}>{job.duration_ms} ms</span>
                      <span style={{
                        padding: '4px 8px',
                        borderRadius: '4px',
                        fontSize: '12px',
                        fontWeight: '600',
                        backgroundColor: job.state === 'Completed' ? '#238636' : (job.state === 'Queued' ? '#1f6feb' : '#da3637'),
                        color: '#fff'
                      }}>{job.state}</span>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {activeTab === 'reports' && (
            <div>
              <h2 style={{ color: '#fff', marginTop: 0 }}>Report Downloads</h2>
              <p style={{ color: '#8b949e' }}>Download static scan results using content-negotiated format exporters.</p>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', marginTop: '20px' }}>
                {executions.filter(e => e.state === 'Completed').map(job => (
                  <div key={job.job_id} style={{
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    padding: '16px',
                    borderRadius: '8px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center'
                  }}>
                    <div>
                      <div style={{ fontWeight: 'bold', color: '#fff' }}>{job.plugin_id}</div>
                      <div style={{ fontSize: '12px', color: '#8b949e', marginTop: '4px' }}>Job ID: {job.job_id}</div>
                    </div>
                    <div style={{ display: 'flex', gap: '8px' }}>
                      {['JSON', 'HTML', 'Markdown', 'SARIF'].map(fmt => (
                        <button
                          key={fmt}
                          onClick={() => triggerNotification(`Downloading report in ${fmt} format...`)}
                          style={{
                            padding: '6px 12px',
                            border: '1px solid #30363d',
                            backgroundColor: '#21262d',
                            color: '#c9d1d9',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px'
                          }}
                        >
                          {fmt}
                        </button>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
