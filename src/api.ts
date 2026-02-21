import { invoke } from '@tauri-apps/api/tauri'
import type { DiscoveryRecord, GroupedRecords, AgentType, RefreshAgent, GroupByKey } from './types'

const INVOKE = {
  discoverAll: 'discover_all' as const,
  refresh: 'refresh' as const,
  groupBy: 'group_by' as const,
  resolveDuplicates: 'resolve_duplicates' as const,
  openPath: 'open_path' as const,
}

export const discoverAll = async (): Promise<DiscoveryRecord[]> => {
  return invoke<DiscoveryRecord[]>(INVOKE.discoverAll)
}

export const refresh = async (agent?: RefreshAgent): Promise<DiscoveryRecord[]> => {
  return invoke<DiscoveryRecord[]>(INVOKE.refresh, { agent: agent && agent !== 'all' ? agent : undefined })
}

export const groupBy = async (records: DiscoveryRecord[], key: GroupByKey): Promise<GroupedRecords[]> => {
  return invoke<GroupedRecords[]>(INVOKE.groupBy, { records, key })
}

export const resolveDuplicates = async (records: DiscoveryRecord[]): Promise<DiscoveryRecord[][]> => {
  return invoke<DiscoveryRecord[][]>(INVOKE.resolveDuplicates, { records })
}

export const openPath = async (path: string): Promise<void> => {
  await invoke<void>(INVOKE.openPath, { path })
}
