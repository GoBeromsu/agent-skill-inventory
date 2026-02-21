export type AgentType = 'codex' | 'claude' | 'gemini' | 'antigravity'
export type ItemKind = 'mcp' | 'skill'
export type ScopeKind = 'global' | 'personal' | 'project' | 'managed' | 'system' | 'session' | 'antigravity-config' | 'unknown'
export type StatusKind = 'enabled' | 'disabled' | 'unknown'
export type GroupByKey = 'agent' | 'scope' | 'location'
export type RefreshAgent = AgentType | 'all'

export interface DiscoverySource {
  kind: string
  file_path: string
  extractor_version: string
  last_modified: number
  file_hash?: string | null
}

export interface DiscoveryDetails {
  command?: string
  args?: string[]
  url?: string
  env?: Record<string, string>
  file?: string
  transport?: string
  description?: string
}

export interface DiscoveryRecord {
  id: string
  agent: AgentType
  kind: ItemKind
  name: string
  location: string
  scope: ScopeKind
  scope_hint: string
  status: StatusKind
  source: DiscoverySource
  details: DiscoveryDetails
  raw: string
  fingerprint: string
  canonical_group_id: string
}

export interface GroupedRecords {
  key: string
  items: DiscoveryRecord[]
}
