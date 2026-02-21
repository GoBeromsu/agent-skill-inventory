import { useEffect, useMemo, useState } from 'react'
import { Sun, Moon, RefreshCw, FolderOpen, ExternalLink } from 'lucide-react'
import type { DiscoveryRecord, AgentType, ScopeKind, ItemKind, StatusKind } from './types'
import { discoverAll, refresh, openPath } from './api'

const AGENTS: AgentType[] = ['codex', 'claude', 'gemini', 'antigravity']
const SCOPES: ScopeKind[] = ['global', 'personal', 'project', 'managed', 'system', 'session', 'antigravity-config', 'unknown']

export type CategoryKind = 'MCP' | 'Skill' | 'Subagent' | 'Soul'

function classifyRecord(record: DiscoveryRecord): CategoryKind {
  if (record.kind === 'mcp') return 'MCP'
  if (record.kind === 'soul') return 'Soul'

  const name = record.name.toLowerCase()
  const loc = record.location.toLowerCase()

  if (name.endsWith('.md') && (name.includes('claude') || name.includes('agent') || name.includes('system') || name.includes('personal') || name.includes('project'))) {
    return 'Soul'
  }

  if (name.includes('subagent') || loc.includes('subagents') || loc.includes('subagent')) {
    return 'Subagent'
  }

  return 'Skill'
}

function maskValue(value: string, show: boolean): string {
  if (!value) return value
  if (show) return value
  if (value.length <= 6) return '***'
  return `${value.slice(0, 3)}***${value.slice(-3)}`
}

function formatTime(value: number): string {
  if (!value) return '-'
  return new Date(value * 1000).toLocaleString('en-US', {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
  })
}

function categoryBadgeClass(category: CategoryKind): string {
  switch (category) {
    case 'MCP': return 'badge-mcp'
    case 'Skill': return 'badge-skill'
    case 'Subagent': return 'badge-subagent'
    case 'Soul': return 'badge-soul'
    default: return 'badge-default'
  }
}

function statusBadgeClass(status: StatusKind): string {
  if (status === 'enabled') return 'status enabled'
  if (status === 'disabled') return 'status disabled'
  return 'status unknown'
}

export default function App() {
  const [records, setRecords] = useState<(DiscoveryRecord & { category: CategoryKind })[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  
  const [query, setQuery] = useState('')
  const [categoryFilter, setCategoryFilter] = useState<'all' | CategoryKind>('all')
  const [agentFilter, setAgentFilter] = useState<'all' | AgentType>('all')
  const [scopeFilter, setScopeFilter] = useState<'all' | ScopeKind>('all')
  const [showOnlyDuplicate, setShowOnlyDuplicate] = useState(false)
  const [showSensitive, setShowSensitive] = useState(false)
  
  const [selected, setSelected] = useState<(DiscoveryRecord & { category: CategoryKind }) | null>(null)

  const [isDark, setIsDark] = useState(() => {
    return window.matchMedia ? window.matchMedia('(prefers-color-scheme: dark)').matches : false;
  });

  useEffect(() => {
    if (isDark) {
      document.body.classList.add('dark')
    } else {
      document.body.classList.remove('dark')
    }
  }, [isDark])

  const processData = (data: DiscoveryRecord[]) => {
    return data.map(r => ({ ...r, category: classifyRecord(r) }))
  }

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const data = await discoverAll()
      const processed = processData(data)
      setRecords(processed)
      setSelected(processed[0] ?? null)
    } catch (err) {
      setError((err as Error).toString())
    } finally {
      setLoading(false)
    }
  }

  const refreshAction = async (agent?: AgentType) => {
    setLoading(true)
    setError(null)
    try {
      const data = await refresh(agent)
      const processed = processData(data)
      setRecords(processed)
      setSelected(processed[0] ?? null)
    } catch (err) {
      setError((err as Error).toString())
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    void load()
  }, [])

  const duplicateCountByGroup = useMemo(() => {
    const counter = new Map<string, number>()
    for (const record of records) {
      const value = counter.get(record.canonical_group_id) ?? 0
      counter.set(record.canonical_group_id, value + 1)
    }
    return counter
  }, [records])

  const categoryCounts = useMemo(() => {
    const map = new Map<CategoryKind | 'all', number>([['all', 0], ['MCP', 0], ['Skill', 0], ['Subagent', 0], ['Soul', 0]])
    for (const record of records) {
      map.set('all', (map.get('all') ?? 0) + 1)
      map.set(record.category, (map.get(record.category) ?? 0) + 1)
    }
    return map
  }, [records])

  const agentCounts = useMemo(() => {
    const map = new Map<AgentType | 'all', number>([['all', 0], ['codex', 0], ['claude', 0], ['gemini', 0], ['antigravity', 0]])
    for (const record of records) {
      map.set('all', (map.get('all') ?? 0) + 1)
      map.set(record.agent, (map.get(record.agent) ?? 0) + 1)
    }
    return map
  }, [records])

  const scopeCounts = useMemo(() => {
    const map = new Map<ScopeKind | 'all', number>([['all', 0]])
    for (const scope of SCOPES) map.set(scope, 0)
    for (const record of records) {
      map.set('all', (map.get('all') ?? 0) + 1)
      map.set(record.scope, (map.get(record.scope) ?? 0) + 1)
    }
    return map
  }, [records])

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    return records.filter((r) => {
      if (categoryFilter !== 'all' && r.category !== categoryFilter) return false
      if (agentFilter !== 'all' && r.agent !== agentFilter) return false
      if (scopeFilter !== 'all' && r.scope !== scopeFilter) return false
      if (showOnlyDuplicate && (duplicateCountByGroup.get(r.canonical_group_id) ?? 0) < 2) return false
      
      if (!q) return true
      const haystack = [r.name, r.location, r.scope_hint, r.agent, r.category, r.scope].join(' ').toLowerCase()
      return haystack.includes(q)
    }).sort((a, b) => {
      const dupA = duplicateCountByGroup.get(a.canonical_group_id) ?? 0
      const dupB = duplicateCountByGroup.get(b.canonical_group_id) ?? 0
      if (dupA !== dupB && showOnlyDuplicate) return dupB - dupA 
      return a.name.localeCompare(b.name)
    })
  }, [records, categoryFilter, agentFilter, scopeFilter, query, showOnlyDuplicate, duplicateCountByGroup])

  if (loading) {
    return (
      <main className="container layout-center">
        <div className="loader-box">
          <div className="spinner"></div>
          <p className="text-muted text-sm">로딩중...</p>
        </div>
      </main>
    )
  }

  return (
    <main className="container layout-main">
      {/* SIDEBAR */}
      <aside className="sidebar">
        <header className="sidebar-header">
          <h1>Inventory</h1>
          <div className="actions">
            <button className="icon-btn" onClick={() => setIsDark(!isDark)} title="Toggle Theme">
              {isDark ? <Moon size={16} /> : <Sun size={16} />}
            </button>
            <button className="icon-btn" onClick={() => refreshAction(agentFilter === 'all' ? undefined : agentFilter)} title="Refresh Data">
              <RefreshCw size={16} />
            </button>
          </div>
        </header>

        <div className="search-box">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="이름, 경로 검색..."
          />
        </div>

        <div className="filter-section">
          <h3>Category</h3>
          <div className="filter-list">
            <button className={categoryFilter === 'all' ? 'active' : ''} onClick={() => setCategoryFilter('all')}>
              전체 <span>{categoryCounts.get('all')}</span>
            </button>
            {(['Soul', 'Subagent', 'MCP', 'Skill'] as CategoryKind[]).map(cat => (
              <button key={cat} className={categoryFilter === cat ? 'active' : ''} onClick={() => setCategoryFilter(cat)}>
                {cat} <span>{categoryCounts.get(cat)}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="filter-section">
          <h3>Agent</h3>
          <div className="filter-list">
            <button className={agentFilter === 'all' ? 'active' : ''} onClick={() => setAgentFilter('all')}>
              전체 <span>{agentCounts.get('all')}</span>
            </button>
            {AGENTS.map(agent => (
              <button key={agent} className={agentFilter === agent ? 'active' : ''} onClick={() => setAgentFilter(agent)}>
                {agent} <span>{agentCounts.get(agent)}</span>
              </button>
            ))}
          </div>
        </div>
        
        <div className="filter-section">
          <h3>Scope</h3>
          <div className="filter-list">
            <button className={scopeFilter === 'all' ? 'active' : ''} onClick={() => setScopeFilter('all')}>
              전체 <span>{scopeCounts.get('all')}</span>
            </button>
            {SCOPES.map(scope => (
              <button key={scope} className={scopeFilter === scope ? 'active' : ''} onClick={() => setScopeFilter(scope)}>
                {scope} <span>{scopeCounts.get(scope)}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="filter-section toggles">
          <label>
            <input type="checkbox" checked={showOnlyDuplicate} onChange={e => setShowOnlyDuplicate(e.target.checked)} />
            중복 항목
          </label>
          <label>
            <input type="checkbox" checked={showSensitive} onChange={e => setShowSensitive(e.target.checked)} />
            Secret 표시
          </label>
        </div>
      </aside>

      {/* MID PANEL */}
      <section className="list-pane">
        <header className="list-header">
          <h2>데이터 <span className="text-muted">({filtered.length})</span></h2>
        </header>
        
        {error && <div className="error-banner">{error}</div>}

        <div className="table-container">
          <table className="data-table">
            <thead>
              <tr>
                <th>도구 (이름)</th>
                <th style={{ width: '80px' }}>분류</th>
                <th style={{ width: '60px' }}>상태</th>
                <th>위치</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map(record => {
                const dupCount = duplicateCountByGroup.get(record.canonical_group_id) ?? 0
                const isDup = dupCount > 1
                const isSelected = selected?.id === record.id
                return (
                  <tr 
                    key={record.id} 
                    className={`${isSelected ? 'selected' : ''} ${isDup ? 'duplicate-row' : ''}`}
                    onClick={() => setSelected(record)}
                  >
                    <td>
                      <div className="cell-name">
                        <span className="agent-indicator" title={record.agent}>{record.agent.charAt(0).toUpperCase()}</span>
                        <strong>{record.name}</strong>
                        {isDup && <span className="badge-duplicate" title="Duplicate items found">{dupCount}</span>}
                      </div>
                    </td>
                    <td><span className={`badge ${categoryBadgeClass(record.category)}`}>{record.category}</span></td>
                    <td><span className={statusBadgeClass(record.status)} title={record.status} /></td>
                    <td className="cell-path" title={record.location}>{record.scope_hint}</td>
                  </tr>
                )
              })}
              {filtered.length === 0 && (
                <tr>
                  <td colSpan={4} className="empty-state">항목이 없습니다.</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>

      {/* RIGHT PANEL */}
      <aside className="detail-pane">
        {selected ? (
          <div className="detail-content">
            <header className="detail-header">
              <div className="title-row">
                <h2>{selected.name}</h2>
                <div className="actions">
                  <button className="icon-btn" onClick={() => void openPath(selected.location)} title="Open in File Explorer">
                    <FolderOpen size={16} />
                  </button>
                  {selected.details.url && (
                    <button className="icon-btn" onClick={() => window.open(selected.details.url, '_blank')} title={`Open Details URL: ${selected.details.url}`}>
                      <ExternalLink size={16} />
                    </button>
                  )}
                </div>
              </div>
              <div className="detail-badges">
                <span className={`badge ${categoryBadgeClass(selected.category)}`}>{selected.category}</span>
                <span className={`badge badge-default`}>{selected.agent}</span>
              </div>
            </header>

            <section className="detail-body">
              <div className="info-grid">
                <span>Scope</span><strong>{selected.scope}</strong>
                <span>Source</span><strong>{selected.source.kind}</strong>
                <span>Updated</span><strong>{formatTime(selected.source.last_modified)}</strong>
              </div>

              <div className="detail-group">
                <h3>Location</h3>
                <div className="code-block" title={selected.location}>{selected.location}</div>
              </div>

              <div className="detail-group">
                <h3>Details</h3>
                <div className="info-grid">
                  <span>CMD</span><strong>{selected.details.command || '-'}</strong>
                  <span>ARGS</span><strong>{selected.details.args?.join(' ') || '-'}</strong>
                </div>
              </div>

              {selected.details.env && Object.keys(selected.details.env).length > 0 && (
                <div className="detail-group">
                  <h3>Environment Variables</h3>
                  <div className="env-list">
                    {Object.entries(selected.details.env).map(([k, v]) => (
                      <div className="env-item" key={k}>
                        <span className="env-key">{k}</span>
                        <code className="env-val">{maskValue(v, showSensitive)}</code>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {selected.details.description && (
                <div className="detail-group">
                  <h3>Description</h3>
                  <p className="desc-text">{selected.details.description}</p>
                </div>
              )}
              
              <div className="detail-group">
                <h3>Raw JSON</h3>
                <pre className="raw-block">{selected.raw || 'N/A'}</pre>
              </div>
            </section>
          </div>
        ) : (
          <div className="empty-state">목록에서 항목을 선택하세요.</div>
        )}
      </aside>
    </main>
  )
}
