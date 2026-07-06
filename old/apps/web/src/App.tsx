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

interface MarketplacePackage {
  id: string;
  name: string;
  display_name: string;
  version: string;
  description: string;
  author: string;
  publisher_id: string | null;
  categories: string;
  tags: string;
  capabilities: string;
  download_url: string;
  checksum: string;
  download_count: number;
  rating_avg: number;
  rating_count: number;
  license: string;
  repository_url: string | null;
  documentation_url: string | null;
  is_featured: boolean;
  is_verified: boolean;
}

interface PublisherProfile {
  id: string;
  name: string;
  display_name: string;
  bio: string | null;
  website: string | null;
  verified: boolean;
  plugin_count: number;
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

  // Toast notification
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'info' | 'error' | 'warning' } | null>(null);

  // Marketplace states
  const [marketplacePackages, setMarketplacePackages] = useState<MarketplacePackage[]>([]);
  const [marketplacePublishers, setMarketplacePublishers] = useState<PublisherProfile[]>([]);
  const [marketplaceSearch, setMarketplaceSearch] = useState('');
  const [marketplaceCategory, setMarketplaceCategory] = useState('');
  const [marketplaceLoading, setMarketplaceLoading] = useState(false);
  const [selectedMktPackage, setSelectedMktPackage] = useState<MarketplacePackage | null>(null);

  // Auth & Collaboration states
  const [currentUser, setCurrentUser] = useState<{ id: string; username: string; role: string } | null>(null);
  const [loginUsername, setLoginUsername] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  const [regUsername, setRegUsername] = useState('');
  const [regPassword, setRegPassword] = useState('');
  const [regRole, setRegRole] = useState('Viewer');
  const [isRegistering, setIsRegistering] = useState(false);

  const [teams, setTeams] = useState<{ id: string; name: string }[]>([]);
  const [teamMembers, setTeamMembers] = useState<{ [teamId: string]: [any, string][] }>({});
  const [newTeamName, setNewTeamName] = useState('');
  const [newMemberUserId, setNewMemberUserId] = useState('');
  const [newMemberRole, setNewMemberRole] = useState('Viewer');
  const [allUsers, setAllUsers] = useState<{ id: string; username: string; role: string }[]>([]);

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
      // 1. Fetch current profile
      const meRes = await fetch(`${API_BASE}/api/v1/auth/me`, { headers: getHeaders() });
      if (meRes.ok) {
        setCurrentUser(await meRes.json());
      } else {
        // Clear token on auth failure
        setAuthToken('');
        setCurrentUser(null);
        return;
      }

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

      // Teams
      const teamsRes = await fetch(`${API_BASE}/api/v1/teams`, { headers: getHeaders() });
      if (teamsRes.ok) {
        const teamsData = await teamsRes.json();
        setTeams(teamsData);
        // Fetch members for each team
        for (const t of teamsData) {
          const memRes = await fetch(`${API_BASE}/api/v1/teams/${t.id}/members`, { headers: getHeaders() });
          if (memRes.ok) {
            const memData = await memRes.json();
            setTeamMembers(prev => ({ ...prev, [t.id]: memData }));
          }
        }
      }

      // Users
      const usersRes = await fetch(`${API_BASE}/api/v1/auth/users`, { headers: getHeaders() });
      if (usersRes.ok) {
        setAllUsers(await usersRes.json());
      }
    } catch (err) {
      showToast('Connection to server failed. Verify API token.', 'error');
    }
  };

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const res = await fetch(`${API_BASE}/api/v1/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: loginUsername, password: loginPassword })
      });
      if (res.ok) {
        const data = await res.json();
        setAuthToken(data.token);
        setCurrentUser(data.user);
        showToast(`Welcome back, ${data.user.username}!`, 'success');
      } else {
        const errText = await res.text();
        showToast(errText || 'Invalid credentials', 'error');
      }
    } catch {
      showToast('Login connection failed', 'error');
    }
  };

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const res = await fetch(`${API_BASE}/api/v1/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username: regUsername, password: regPassword, role: regRole })
      });
      if (res.ok) {
        showToast('Registered successfully! Please log in.', 'success');
        setIsRegistering(false);
        setLoginUsername(regUsername);
        setLoginPassword('');
      } else {
        const errText = await res.text();
        showToast(errText || 'Registration failed', 'error');
      }
    } catch {
      showToast('Registration connection failed', 'error');
    }
  };

  const handleLogout = () => {
    setAuthToken('');
    setCurrentUser(null);
    localStorage.removeItem('pg_auth_token');
    showToast('Logged out successfully', 'info');
  };

  const handleCreateTeam = async () => {
    if (!newTeamName.trim()) return;
    try {
      const res = await fetch(`${API_BASE}/api/v1/teams`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ name: newTeamName })
      });
      if (res.ok) {
        showToast('Team created successfully', 'success');
        setNewTeamName('');
        fetchData();
      } else {
        const text = await res.text();
        showToast(text || 'Failed to create team', 'error');
      }
    } catch {
      showToast('Error creating team', 'error');
    }
  };

  const handleAddTeamMember = async (teamId: string) => {
    if (!newMemberUserId) return;
    try {
      const res = await fetch(`${API_BASE}/api/v1/teams/${teamId}/members`, {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ user_id: newMemberUserId, role: newMemberRole })
      });
      if (res.ok) {
        showToast('Member added/updated successfully', 'success');
        fetchData();
      } else {
        const text = await res.text();
        showToast(text || 'Failed to add member', 'error');
      }
    } catch {
      showToast('Error adding member', 'error');
    }
  };

  const handleRemoveTeamMember = async (teamId: string, userId: string) => {
    try {
      const res = await fetch(`${API_BASE}/api/v1/teams/${teamId}/members/${userId}`, {
        method: 'DELETE',
        headers: getHeaders()
      });
      if (res.ok) {
        showToast('Member removed successfully', 'success');
        fetchData();
      }
    } catch {
      showToast('Error removing member', 'error');
    }
  };

  const fetchMarketplace = async (q = '', cat = '') => {
    try {
      setMarketplaceLoading(true);
      const url = q
        ? `${API_BASE}/api/v1/marketplace/search?q=${encodeURIComponent(q)}${cat ? `&category=${encodeURIComponent(cat)}` : ''}`
        : `${API_BASE}/api/v1/marketplace`;
      const res = await fetch(url, { headers: getHeaders() });
      if (res.ok) setMarketplacePackages(await res.json());
      const pubRes = await fetch(`${API_BASE}/api/v1/marketplace/publishers`, { headers: getHeaders() });
      if (pubRes.ok) setMarketplacePublishers(await pubRes.json());
    } catch (err) {
      showToast('Failed to load marketplace', 'error');
    } finally {
      setMarketplaceLoading(false);
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
    if (currentUser?.role === 'Viewer') {
      showToast('Viewer role cannot add targets', 'error');
      return;
    }
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
    if (currentUser?.role !== 'Owner') {
      showToast('Only Owners can remove targets', 'error');
      return;
    }
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
    if (currentUser?.role === 'Viewer') {
      showToast('Viewer role cannot toggle plugins', 'error');
      return;
    }
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
    if (currentUser?.role === 'Viewer') {
      showToast('Viewer role cannot launch scans', 'error');
      return;
    }
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

  if (!authToken) {
    return (
      <div style={{
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        backgroundColor: '#0d1117',
        color: '#c9d1d9',
        minHeight: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'radial-gradient(circle at center, #1f2937, #0d1117)'
      }}>
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
          }}>
            {toast.message}
          </div>
        )}
        <div style={{
          backgroundColor: '#161b22',
          border: '1px solid #30363d',
          padding: '40px',
          borderRadius: '12px',
          width: '100%',
          maxWidth: '420px',
          boxShadow: '0 8px 24px rgba(0,0,0,0.3)'
        }}>
          <div style={{ textAlign: 'center', marginBottom: '32px' }}>
            <div style={{
              background: 'linear-gradient(135deg, #58a6ff, #1f6feb)',
              width: '64px',
              height: '64px',
              borderRadius: '16px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontWeight: 'bold',
              fontSize: '32px',
              color: '#fff',
              margin: '0 auto 16px auto'
            }}>PG</div>
            <h2 style={{ color: '#fff', margin: 0 }}>PolyGlid Workspace</h2>
            <p style={{ color: '#8b949e', fontSize: '14px', marginTop: '6px' }}>Secure WebAssembly Security Framework</p>
          </div>

          {isRegistering ? (
            <form onSubmit={handleRegister}>
              <h3 style={{ color: '#fff', marginTop: 0, marginBottom: '20px' }}>Register User</h3>
              <div style={{ marginBottom: '16px' }}>
                <label style={{ display: 'block', color: '#c9d1d9', fontSize: '14px', marginBottom: '6px' }}>Username</label>
                <input
                  type="text"
                  required
                  value={regUsername}
                  onChange={e => setRegUsername(e.target.value)}
                  style={{
                    width: '100%',
                    padding: '10px',
                    backgroundColor: '#0d1117',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    outline: 'none',
                    boxSizing: 'border-box'
                  }}
                />
              </div>
              <div style={{ marginBottom: '16px' }}>
                <label style={{ display: 'block', color: '#c9d1d9', fontSize: '14px', marginBottom: '6px' }}>Password</label>
                <input
                  type="password"
                  required
                  value={regPassword}
                  onChange={e => setRegPassword(e.target.value)}
                  style={{
                    width: '100%',
                    padding: '10px',
                    backgroundColor: '#0d1117',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    outline: 'none',
                    boxSizing: 'border-box'
                  }}
                />
              </div>
              <div style={{ marginBottom: '24px' }}>
                <label style={{ display: 'block', color: '#c9d1d9', fontSize: '14px', marginBottom: '6px' }}>Requested Role</label>
                <select
                  value={regRole}
                  onChange={e => setRegRole(e.target.value)}
                  style={{
                    width: '100%',
                    padding: '10px',
                    backgroundColor: '#0d1117',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    outline: 'none',
                    cursor: 'pointer'
                  }}
                >
                  <option value="Viewer">Viewer (Read-only)</option>
                  <option value="Editor">Editor (Run scans, configure)</option>
                  <option value="Owner">Owner (Full access)</option>
                </select>
                <span style={{ fontSize: '12px', color: '#8b949e', display: 'block', marginTop: '6px' }}>
                  Note: The first user registered is automatically an Owner. Subsequent registrations are created as Viewers unless authorized.
                </span>
              </div>
              <button
                type="submit"
                style={{
                  width: '100%',
                  padding: '12px',
                  backgroundColor: '#238636',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  fontSize: '14px',
                  fontWeight: '600',
                  marginBottom: '16px'
                }}
              >
                Register
              </button>
              <div style={{ textAlign: 'center', fontSize: '14px' }}>
                <span style={{ color: '#8b949e' }}>Already have an account? </span>
                <button
                  type="button"
                  onClick={() => setIsRegistering(false)}
                  style={{ background: 'none', border: 'none', color: '#58a6ff', cursor: 'pointer', padding: 0 }}
                >
                  Log in
                </button>
              </div>
            </form>
          ) : (
            <form onSubmit={handleLogin}>
              <h3 style={{ color: '#fff', marginTop: 0, marginBottom: '20px' }}>Sign In</h3>
              <div style={{ marginBottom: '16px' }}>
                <label style={{ display: 'block', color: '#c9d1d9', fontSize: '14px', marginBottom: '6px' }}>Username</label>
                <input
                  type="text"
                  required
                  value={loginUsername}
                  onChange={e => setLoginUsername(e.target.value)}
                  style={{
                    width: '100%',
                    padding: '10px',
                    backgroundColor: '#0d1117',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    outline: 'none',
                    boxSizing: 'border-box'
                  }}
                />
              </div>
              <div style={{ marginBottom: '24px' }}>
                <label style={{ display: 'block', color: '#c9d1d9', fontSize: '14px', marginBottom: '6px' }}>Password</label>
                <input
                  type="password"
                  required
                  value={loginPassword}
                  onChange={e => setLoginPassword(e.target.value)}
                  style={{
                    width: '100%',
                    padding: '10px',
                    backgroundColor: '#0d1117',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    outline: 'none',
                    boxSizing: 'border-box'
                  }}
                />
              </div>
              <button
                type="submit"
                style={{
                  width: '100%',
                  padding: '12px',
                  backgroundColor: '#1f6feb',
                  color: '#fff',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer',
                  fontSize: '14px',
                  fontWeight: '600',
                  marginBottom: '16px'
                }}
              >
                Log In
              </button>
              <div style={{ textAlign: 'center', fontSize: '14px' }}>
                <span style={{ color: '#8b949e' }}>Don't have an account? </span>
                <button
                  type="button"
                  onClick={() => setIsRegistering(true)}
                  style={{ background: 'none', border: 'none', color: '#58a6ff', cursor: 'pointer', padding: 0 }}
                >
                  Create one
                </button>
              </div>
            </form>
          )}
        </div>
      </div>
    );
  }

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
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          {currentUser && (
            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', backgroundColor: '#161b22', border: '1px solid #30363d', padding: '6px 12px', borderRadius: '20px' }}>
              <span style={{ fontSize: '14px', color: '#c9d1d9', fontWeight: '500' }}>👤 {currentUser.username}</span>
              <span style={{
                fontSize: '11px',
                fontWeight: 'bold',
                padding: '2px 8px',
                borderRadius: '12px',
                color: '#fff',
                backgroundColor: currentUser.role === 'Owner' ? '#da3637' : (currentUser.role === 'Editor' ? '#d97706' : '#8b949e')
              }}>{currentUser.role}</span>
            </div>
          )}
          <button
            onClick={handleLogout}
            style={{
              padding: '8px 16px',
              borderRadius: '6px',
              border: '1px solid #da3637',
              backgroundColor: 'transparent',
              color: '#da3637',
              fontWeight: '600',
              cursor: 'pointer',
              fontSize: '13px',
              transition: 'all 0.2s ease'
            }}
            onMouseEnter={e => {
              e.currentTarget.style.backgroundColor = '#da3637';
              e.currentTarget.style.color = '#fff';
            }}
            onMouseLeave={e => {
              e.currentTarget.style.backgroundColor = 'transparent';
              e.currentTarget.style.color = '#da3637';
            }}
          >
            Sign Out
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
            { id: 'dashboard', label: '🖥 Dashboard' },
            { id: 'plugins', label: '🔌 Plugin Manager' },
            { id: 'targets', label: '🎯 Target Manager' },
            { id: 'execution', label: '🚀 Scan Launcher' },
            { id: 'monitor', label: '📡 Live Monitor' },
            { id: 'reports', label: '📄 Reports' },
            { id: 'marketplace', label: '🛒 Marketplace' },
            { id: 'collaboration', label: '👥 Team Collaboration' },
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

          {activeTab === 'marketplace' && (
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '28px' }}>
                <div>
                  <h2 style={{ color: '#fff', margin: 0, fontSize: '24px' }}>🛒 Plugin Marketplace</h2>
                  <p style={{ color: '#8b949e', margin: '6px 0 0 0', fontSize: '14px' }}>Discover, install, and manage security plugins from the community registry</p>
                </div>
                <button
                  onClick={() => fetchMarketplace(marketplaceSearch, marketplaceCategory)}
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#238636',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '14px',
                    fontWeight: '600'
                  }}
                >
                  ↻ Refresh
                </button>
              </div>

              {/* Search Bar */}
              <div style={{ display: 'flex', gap: '12px', marginBottom: '28px' }}>
                <input
                  type="text"
                  value={marketplaceSearch}
                  onChange={e => setMarketplaceSearch(e.target.value)}
                  onKeyDown={e => e.key === 'Enter' && fetchMarketplace(marketplaceSearch, marketplaceCategory)}
                  placeholder="Search plugins... (e.g. recon, port scanner, dns)"
                  style={{
                    flex: 1,
                    padding: '12px 16px',
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    outline: 'none'
                  }}
                />
                <select
                  value={marketplaceCategory}
                  onChange={e => setMarketplaceCategory(e.target.value)}
                  style={{
                    padding: '12px 16px',
                    backgroundColor: '#161b22',
                    border: '1px solid #30363d',
                    borderRadius: '6px',
                    color: '#c9d1d9',
                    fontSize: '14px',
                    cursor: 'pointer',
                    minWidth: '160px'
                  }}
                >
                  <option value="">All Categories</option>
                  <option value="security">Security</option>
                  <option value="recon">Recon</option>
                  <option value="scanning">Scanning</option>
                  <option value="reporting">Reporting</option>
                  <option value="utilities">Utilities</option>
                </select>
                <button
                  onClick={() => fetchMarketplace(marketplaceSearch, marketplaceCategory)}
                  style={{
                    padding: '12px 24px',
                    backgroundColor: '#1f6feb',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    fontSize: '14px',
                    fontWeight: '600'
                  }}
                >
                  🔍 Search
                </button>
              </div>

              {marketplaceLoading && (
                <div style={{ textAlign: 'center', color: '#8b949e', padding: '40px' }}>Loading marketplace...</div>
              )}

              {/* Package Detail View */}
              {selectedMktPackage && !marketplaceLoading && (
                <div>
                  <button
                    onClick={() => setSelectedMktPackage(null)}
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
                    ← Back to Marketplace
                  </button>
                  <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', borderRadius: '8px', padding: '28px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                      <div>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '8px' }}>
                          <h2 style={{ color: '#fff', margin: 0 }}>{selectedMktPackage.display_name}</h2>
                          {selectedMktPackage.is_verified && (
                            <span style={{ backgroundColor: '#1f6feb', color: '#fff', padding: '2px 8px', borderRadius: '12px', fontSize: '12px' }}>✓ Verified</span>
                          )}
                          {selectedMktPackage.is_featured && (
                            <span style={{ backgroundColor: '#9a3412', color: '#fff', padding: '2px 8px', borderRadius: '12px', fontSize: '12px' }}>⭐ Featured</span>
                          )}
                        </div>
                        <p style={{ color: '#8b949e', margin: '0 0 8px 0' }}>by {selectedMktPackage.author} · v{selectedMktPackage.version} · {selectedMktPackage.license}</p>
                        <p style={{ color: '#c9d1d9', margin: 0 }}>{selectedMktPackage.description}</p>
                      </div>
                      <button
                        onClick={async () => {
                          if (currentUser?.role !== 'Owner') {
                            showToast('Only Owners can install marketplace packages', 'error');
                            return;
                          }
                          try {
                            const res = await fetch(`${API_BASE}/api/v1/marketplace/packages/${selectedMktPackage.id}/install`, {
                              method: 'POST',
                              headers: getHeaders(),
                              body: JSON.stringify({ plugin_id: null })
                            });
                            if (res.ok) {
                              showToast(`Package '${selectedMktPackage.display_name}' install recorded!`, 'success');
                              fetchMarketplace(marketplaceSearch, marketplaceCategory);
                            } else {
                              showToast('Install failed', 'error');
                            }
                          } catch {
                            showToast('Install error', 'error');
                          }
                        }}
                        style={{
                          padding: '12px 28px',
                          backgroundColor: '#238636',
                          color: '#fff',
                          border: 'none',
                          borderRadius: '6px',
                          cursor: 'pointer',
                          fontWeight: '600',
                          fontSize: '14px',
                          whiteSpace: 'nowrap'
                        }}
                      >
                        ⬇ Install Plugin
                      </button>
                    </div>

                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '16px', marginTop: '24px' }}>
                      {[
                        { label: 'Downloads', value: selectedMktPackage.download_count.toLocaleString() },
                        { label: 'Rating', value: selectedMktPackage.rating_count > 0 ? `${selectedMktPackage.rating_avg.toFixed(1)} ★ (${selectedMktPackage.rating_count})` : 'No ratings' },
                        { label: 'License', value: selectedMktPackage.license },
                        { label: 'Checksum', value: selectedMktPackage.checksum.substring(0, 20) + '...' },
                      ].map(stat => (
                        <div key={stat.label} style={{ backgroundColor: '#0d1117', border: '1px solid #30363d', padding: '16px', borderRadius: '6px' }}>
                          <div style={{ color: '#8b949e', fontSize: '12px', marginBottom: '6px' }}>{stat.label}</div>
                          <div style={{ color: '#c9d1d9', fontSize: '14px', fontWeight: '500' }}>{stat.value}</div>
                        </div>
                      ))}
                    </div>

                    <div style={{ marginTop: '24px' }}>
                      <h4 style={{ color: '#fff', marginBottom: '12px' }}>Required Capabilities</h4>
                      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                        {(() => {
                          try {
                            const caps = JSON.parse(selectedMktPackage.capabilities);
                            return Array.isArray(caps) ? caps.map((cap: string) => (
                              <span key={cap} style={{
                                backgroundColor: '#161b22',
                                border: '1px solid #388bfd',
                                color: '#58a6ff',
                                padding: '4px 10px',
                                borderRadius: '12px',
                                fontSize: '12px'
                              }}>{cap}</span>
                            )) : <span style={{ color: '#8b949e' }}>None specified</span>;
                          } catch {
                            return <span style={{ color: '#8b949e' }}>None specified</span>;
                          }
                        })()}
                      </div>
                    </div>

                    <div style={{ marginTop: '24px' }}>
                      <h4 style={{ color: '#fff', marginBottom: '12px' }}>Categories & Tags</h4>
                      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                        {(() => {
                          try {
                            const tags = JSON.parse(selectedMktPackage.tags);
                            return Array.isArray(tags) ? tags.map((tag: string) => (
                              <span key={tag} style={{
                                backgroundColor: '#21262d',
                                border: '1px solid #30363d',
                                color: '#8b949e',
                                padding: '3px 8px',
                                borderRadius: '4px',
                                fontSize: '12px'
                              }}>{tag}</span>
                            )) : null;
                          } catch { return null; }
                        })()}
                      </div>
                    </div>

                    {(selectedMktPackage.repository_url || selectedMktPackage.documentation_url) && (
                      <div style={{ marginTop: '24px', display: 'flex', gap: '12px' }}>
                        {selectedMktPackage.repository_url && (
                          <a href={selectedMktPackage.repository_url} target="_blank" rel="noreferrer"
                            style={{ color: '#58a6ff', fontSize: '14px' }}>📂 Repository</a>
                        )}
                        {selectedMktPackage.documentation_url && (
                          <a href={selectedMktPackage.documentation_url} target="_blank" rel="noreferrer"
                            style={{ color: '#58a6ff', fontSize: '14px' }}>📖 Documentation</a>
                        )}
                      </div>
                    )}
                  </div>
                </div>
              )}

              {/* Package Grid */}
              {!selectedMktPackage && !marketplaceLoading && (
                <div>
                  {marketplacePackages.length === 0 ? (
                    <div style={{
                      textAlign: 'center',
                      padding: '60px 20px',
                      backgroundColor: '#161b22',
                      border: '1px solid #30363d',
                      borderRadius: '8px'
                    }}>
                      <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔍</div>
                      <h3 style={{ color: '#fff', margin: '0 0 8px 0' }}>No packages found</h3>
                      <p style={{ color: '#8b949e', margin: 0 }}>
                        Try searching for something, or click Refresh to load the featured packages.
                      </p>
                      <button
                        onClick={() => fetchMarketplace()}
                        style={{
                          marginTop: '20px',
                          padding: '10px 24px',
                          backgroundColor: '#1f6feb',
                          color: '#fff',
                          border: 'none',
                          borderRadius: '6px',
                          cursor: 'pointer'
                        }}
                      >Load Featured Plugins</button>
                    </div>
                  ) : (
                    <>
                      <p style={{ color: '#8b949e', fontSize: '14px', marginBottom: '16px' }}>
                        {marketplacePackages.length} package{marketplacePackages.length !== 1 ? 's' : ''} found
                      </p>
                      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', gap: '16px' }}>
                        {marketplacePackages.map(pkg => (
                          <div
                            key={pkg.id}
                            onClick={() => setSelectedMktPackage(pkg)}
                            style={{
                              backgroundColor: '#161b22',
                              border: `1px solid ${pkg.is_featured ? '#f78166' : '#30363d'}`,
                              borderRadius: '8px',
                              padding: '20px',
                              cursor: 'pointer',
                              transition: 'all 0.2s ease',
                              position: 'relative',
                              overflow: 'hidden'
                            }}
                            onMouseEnter={e => (e.currentTarget.style.borderColor = '#58a6ff')}
                            onMouseLeave={e => (e.currentTarget.style.borderColor = pkg.is_featured ? '#f78166' : '#30363d')}
                          >
                            {pkg.is_featured && (
                              <div style={{
                                position: 'absolute',
                                top: 0,
                                right: 0,
                                backgroundColor: '#9a3412',
                                color: '#fff',
                                fontSize: '10px',
                                padding: '3px 8px',
                                borderBottomLeftRadius: '6px'
                              }}>⭐ FEATURED</div>
                            )}
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '10px' }}>
                              <div>
                                <h3 style={{ color: '#fff', margin: 0, fontSize: '16px' }}>
                                  {pkg.display_name}
                                  {pkg.is_verified && <span style={{ marginLeft: '6px', color: '#1f6feb', fontSize: '12px' }}>✓</span>}
                                </h3>
                                <p style={{ color: '#8b949e', margin: '4px 0 0 0', fontSize: '12px' }}>
                                  {pkg.author} · v{pkg.version}
                                </p>
                              </div>
                              <span style={{
                                backgroundColor: '#21262d',
                                color: '#8b949e',
                                padding: '2px 8px',
                                borderRadius: '4px',
                                fontSize: '11px'
                              }}>{pkg.license}</span>
                            </div>
                            <p style={{
                              color: '#c9d1d9',
                              margin: '0 0 14px 0',
                              fontSize: '13px',
                              lineHeight: '1.5',
                              overflow: 'hidden',
                              display: '-webkit-box',
                              WebkitLineClamp: 2,
                              WebkitBoxOrient: 'vertical',
                            } as React.CSSProperties}>{pkg.description}</p>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '12px' }}>
                              <div style={{ display: 'flex', gap: '12px', color: '#8b949e' }}>
                                <span>⬇ {pkg.download_count.toLocaleString()}</span>
                                <span>{pkg.rating_count > 0 ? `${pkg.rating_avg.toFixed(1)} ★` : 'No ratings'}</span>
                              </div>
                              <span style={{
                                backgroundColor: '#238636',
                                color: '#fff',
                                padding: '4px 10px',
                                borderRadius: '4px',
                                fontSize: '11px'
                              }}>View →</span>
                            </div>
                          </div>
                        ))}
                      </div>
                    </>
                  )}

                  {/* Publishers Section */}
                  {marketplacePublishers.length > 0 && (
                    <div style={{ marginTop: '40px' }}>
                      <h3 style={{ color: '#fff', marginBottom: '16px' }}>📡 Verified Publishers</h3>
                      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(260px, 1fr))', gap: '12px' }}>
                        {marketplacePublishers.map(pub => (
                          <div key={pub.id} style={{
                            backgroundColor: '#161b22',
                            border: '1px solid #30363d',
                            borderRadius: '8px',
                            padding: '16px',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '14px'
                          }}>
                            <div style={{
                              width: '44px',
                              height: '44px',
                              borderRadius: '50%',
                              backgroundColor: '#21262d',
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: 'center',
                              fontSize: '18px',
                              flexShrink: 0
                            }}>
                              {pub.verified ? '🔐' : '👤'}
                            </div>
                            <div>
                              <div style={{ color: '#fff', fontWeight: '600', fontSize: '14px' }}>
                                {pub.display_name}
                                {pub.verified && <span style={{ marginLeft: '6px', color: '#1f6feb', fontSize: '11px' }}>✓ Verified</span>}
                              </div>
                              <div style={{ color: '#8b949e', fontSize: '12px', marginTop: '2px' }}>
                                {pub.plugin_count} plugin{pub.plugin_count !== 1 ? 's' : ''}
                                {pub.website && <> · <a href={pub.website} target="_blank" rel="noreferrer" style={{ color: '#58a6ff' }}>Website</a></>}
                              </div>
                              {pub.bio && <div style={{ color: '#8b949e', fontSize: '12px', marginTop: '4px' }}>{pub.bio}</div>}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}

           {activeTab === 'collaboration' && (
            <div>
              <div style={{ marginBottom: '28px' }}>
                <h2 style={{ color: '#fff', margin: 0, fontSize: '24px' }}>👥 Team Collaboration & Workspaces</h2>
                <p style={{ color: '#8b949e', margin: '6px 0 0 0', fontSize: '14px' }}>Manage workspace teams, member roles, and secure access boundaries</p>
              </div>

              {/* Grid Layout */}
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 2fr', gap: '24px', alignItems: 'start' }}>
                
                {/* Left Side: Create Team & Workspace Members */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
                  
                  {/* Create Team Card */}
                  {(currentUser?.role === 'Owner' || currentUser?.role === 'Editor') && (
                    <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', borderRadius: '8px', padding: '20px' }}>
                      <h3 style={{ color: '#fff', margin: '0 0 16px 0', fontSize: '16px' }}>Create New Team</h3>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                        <input
                          type="text"
                          placeholder="Team Name (e.g. SecOps-Recon)"
                          value={newTeamName}
                          onChange={e => setNewTeamName(e.target.value)}
                          style={{
                            padding: '10px 12px',
                            backgroundColor: '#0d1117',
                            border: '1px solid #30363d',
                            borderRadius: '6px',
                            color: '#c9d1d9',
                            fontSize: '14px',
                            outline: 'none'
                          }}
                        />
                        <button
                          onClick={handleCreateTeam}
                          style={{
                            padding: '10px',
                            backgroundColor: '#238636',
                            color: '#fff',
                            border: 'none',
                            borderRadius: '6px',
                            cursor: 'pointer',
                            fontWeight: '600',
                            fontSize: '14px'
                          }}
                        >
                          Create Team
                        </button>
                      </div>
                    </div>
                  )}

                  {/* Registered Workspace Users List */}
                  <div style={{ backgroundColor: '#161b22', border: '1px solid #30363d', borderRadius: '8px', padding: '20px' }}>
                    <h3 style={{ color: '#fff', margin: '0 0 16px 0', fontSize: '16px' }}>Workspace Users ({allUsers.length})</h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                      {allUsers.map(u => (
                        <div key={u.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', backgroundColor: '#0d1117', padding: '10px 12px', borderRadius: '6px', border: '1px solid #21262d' }}>
                          <span style={{ fontSize: '14px', color: '#c9d1d9', fontWeight: '500' }}>👤 {u.username}</span>
                          <span style={{
                            fontSize: '11px',
                            fontWeight: 'bold',
                            padding: '2px 8px',
                            borderRadius: '10px',
                            color: '#fff',
                            backgroundColor: u.role === 'Owner' ? '#da3637' : (u.role === 'Editor' ? '#d97706' : '#8b949e')
                          }}>{u.role}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Right Side: Active Teams & Team Members */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
                  {teams.length === 0 ? (
                    <div style={{
                      textAlign: 'center',
                      padding: '40px 20px',
                      backgroundColor: '#161b22',
                      border: '1px solid #30363d',
                      borderRadius: '8px',
                      color: '#8b949e'
                    }}>
                      <div style={{ fontSize: '32px', marginBottom: '12px' }}>👥</div>
                      <h3 style={{ color: '#fff', margin: '0 0 6px 0' }}>No Teams Created Yet</h3>
                      <p style={{ margin: 0, fontSize: '14px' }}>
                        Create a team to group users and manage shared capabilities.
                      </p>
                    </div>
                  ) : (
                    teams.map(team => (
                      <div key={team.id} style={{ backgroundColor: '#161b22', border: '1px solid #30363d', borderRadius: '8px', padding: '24px' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid #30363d', paddingBottom: '14px', marginBottom: '16px' }}>
                          <h3 style={{ color: '#fff', margin: 0, fontSize: '18px' }}>🛡 {team.name}</h3>
                          <span style={{ color: '#8b949e', fontSize: '12px' }}>ID: {team.id}</span>
                        </div>

                        {/* Add Member Form (Owners Only) */}
                        {currentUser?.role === 'Owner' && (
                          <div style={{ display: 'flex', gap: '12px', alignItems: 'center', backgroundColor: '#0d1117', padding: '12px', borderRadius: '6px', marginBottom: '16px', border: '1px solid #21262d' }}>
                            <select
                              value={newMemberUserId}
                              onChange={e => setNewMemberUserId(e.target.value)}
                              style={{
                                flex: 2,
                                padding: '8px 12px',
                                backgroundColor: '#161b22',
                                border: '1px solid #30363d',
                                borderRadius: '6px',
                                color: '#c9d1d9',
                                fontSize: '13px',
                                cursor: 'pointer'
                              }}
                            >
                              <option value="">Select User to Add...</option>
                              {allUsers
                                .filter(u => !(teamMembers[team.id] || []).some(m => m[0].id === u.id))
                                .map(u => (
                                  <option key={u.id} value={u.id}>{u.username} ({u.role})</option>
                                ))}
                            </select>
                            <select
                              value={newMemberRole}
                              onChange={e => setNewMemberRole(e.target.value)}
                              style={{
                                flex: 1,
                                padding: '8px 12px',
                                backgroundColor: '#161b22',
                                border: '1px solid #30363d',
                                borderRadius: '6px',
                                color: '#c9d1d9',
                                fontSize: '13px',
                                cursor: 'pointer'
                              }}
                            >
                              <option value="Viewer">Viewer</option>
                              <option value="Editor">Editor</option>
                              <option value="Owner">Owner</option>
                            </select>
                            <button
                              onClick={() => handleAddTeamMember(team.id)}
                              style={{
                                padding: '8px 16px',
                                backgroundColor: '#1f6feb',
                                color: '#fff',
                                border: 'none',
                                borderRadius: '6px',
                                cursor: 'pointer',
                                fontWeight: '600',
                                fontSize: '13px'
                              }}
                            >
                              Add Member
                            </button>
                          </div>
                        )}

                        {/* Members List */}
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                          <h4 style={{ color: '#fff', margin: '0 0 8px 0', fontSize: '13px' }}>Team Members</h4>
                          {(!teamMembers[team.id] || teamMembers[team.id].length === 0) ? (
                            <div style={{ color: '#8b949e', fontSize: '13px', fontStyle: 'italic', padding: '6px 0' }}>No members in this team.</div>
                          ) : (
                            teamMembers[team.id].map(([memberUser, memberRole]) => (
                              <div key={memberUser.id} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', backgroundColor: '#0d1117', padding: '10px 14px', borderRadius: '6px' }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                                  <span style={{ color: '#fff', fontSize: '14px', fontWeight: '500' }}>👤 {memberUser.username}</span>
                                  <span style={{
                                    fontSize: '10px',
                                    fontWeight: 'bold',
                                    padding: '2px 6px',
                                    borderRadius: '8px',
                                    color: '#fff',
                                    backgroundColor: memberRole === 'Owner' ? '#da3637' : (memberRole === 'Editor' ? '#d97706' : '#8b949e')
                                  }}>{memberRole}</span>
                                </div>
                                {currentUser?.role === 'Owner' && (
                                  <button
                                    onClick={() => handleRemoveTeamMember(team.id, memberUser.id)}
                                    style={{
                                      background: 'transparent',
                                      border: 'none',
                                      color: '#f85149',
                                      cursor: 'pointer',
                                      fontSize: '12px',
                                      fontWeight: '600'
                                    }}
                                  >
                                    Remove
                                  </button>
                                )}
                              </div>
                            ))
                          )}
                        </div>
                      </div>
                    ))
                  )}
                </div>
              </div>
            </div>
          )}

        </main>
      </div>
    </div>
  );
}

export default App;
