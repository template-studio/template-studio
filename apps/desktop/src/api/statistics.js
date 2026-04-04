import { invoke } from '@tauri-apps/api/core'

/**
 * 获取统计数据
 * @returns {Promise<{total_projects: number, total_datasources: number, total_languages: number, total_tables: number}>}
 */
export const getStatistics = async () => {
  const result = await invoke('db_get_statistics')
  return JSON.parse(result)
}

/**
 * 获取最近项目列表
 * @param {number} limit - 返回数量限制，默认 5
 * @returns {Promise<Array<{id: number, name: string, description: string|null, database_name: string|null, table_count: number, created_at: string}>>}
 */
export const getRecentProjects = async (limit = 5) => {
  const result = await invoke('db_get_recent_projects', { limit })
  return JSON.parse(result)
}
