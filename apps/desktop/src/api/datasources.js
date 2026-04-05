/**
 * 数据源 API 模块
 *
 * 提供数据源的 CRUD 操作和连接测试功能
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 获取所有数据源
 * @returns {Promise<Array>} 数据源列表
 */
export function getAllDatasources() {
  return invoke('db_get_all_datasources').then(data => JSON.parse(data))
}

/**
 * 根据 ID 获取单个数据源
 * @param {number} id - 数据源 ID
 * @returns {Promise<Object>} 数据源对象
 */
export function getDatasource(id) {
  return invoke('db_get_datasource', { id }).then(data => JSON.parse(data))
}

/**
 * 创建数据源
 * @param {Object} data - 数据源配置
 * @param {string} data.type - 数据库类型 (mysql/postgresql/sqlite)
 * @param {string} data.name - 数据源名称
 * @param {string} [data.host] - 主机地址 (MySQL/PostgreSQL)
 * @param {number} [data.port] - 端口号 (MySQL/PostgreSQL)
 * @param {string} [data.username] - 用户名 (MySQL/PostgreSQL)
 * @param {string} [data.password] - 密码 (MySQL/PostgreSQL)
 * @param {string} [data.database] - 初始数据库 (PostgreSQL, 可选)
 * @param {string} [data.sqliteFile] - SQLite 文件路径 (仅 SQLite)
 * @returns {Promise<number>} 新创建的数据源 ID
 */
export function createDatasource(data) {
  const params = {
    name: data.name,
    type: data.type
  }

  // MySQL/PostgreSQL 特有字段
  if (data.type !== 'sqlite') {
    params.host = data.host || 'localhost'
    params.port = data.port || getDefaultPort(data.type)
    params.username = data.username || ''
    params.password = data.password || ''
    // PostgreSQL 可选的初始数据库
    if (data.type === 'postgresql' && data.database) {
      params.database = data.database
    }
  }

  // SQLite 特有字段
  if (data.type === 'sqlite' && data.sqliteFile) {
    params.sqliteFile = data.sqliteFile
  }

  return invoke('db_create_datasource', { params })
}

/**
 * 更新数据源
 * @param {number} id - 数据源 ID
 * @param {Object} data - 更新的数据源配置
 * @param {string} data.type - 数据库类型 (mysql/postgresql/sqlite)
 * @param {string} data.name - 数据源名称
 * @param {string} [data.host] - 主机地址 (MySQL/PostgreSQL)
 * @param {number} [data.port] - 端口号 (MySQL/PostgreSQL)
 * @param {string} [data.username] - 用户名 (MySQL/PostgreSQL)
 * @param {string} [data.password] - 密码 (MySQL/PostgreSQL)
 * @param {string} [data.database] - 初始数据库 (PostgreSQL, 可选)
 * @param {string} [data.sqliteFile] - SQLite 文件路径 (仅 SQLite)
 * @returns {Promise<void>}
 */
export function updateDatasource(id, data) {
  const params = {
    name: data.name,
    type: data.type
  }

  // MySQL/PostgreSQL 特有字段
  if (data.type !== 'sqlite') {
    params.host = data.host || 'localhost'
    params.port = data.port || getDefaultPort(data.type)
    params.username = data.username || ''
    params.password = data.password || ''
    // PostgreSQL 可选的初始数据库
    if (data.type === 'postgresql' && data.database) {
      params.database = data.database
    }
  }

  // SQLite 特有字段
  if (data.type === 'sqlite' && data.sqliteFile) {
    params.sqliteFile = data.sqliteFile
  }

  return invoke('db_update_datasource', { id, params })
}

/**
 * 删除数据源
 * @param {number} id - 数据源 ID
 * @returns {Promise<void>}
 */
export function deleteDatasource(id) {
  return invoke('db_delete_datasource', { id })
}

/**
 * 测试数据库连接
 * @param {Object} data - 连接配置
 * @param {string} data.type - 数据库类型
 * @param {string} [data.host] - 主机地址 (可选)
 * @param {number} [data.port] - 端口号 (可选)
 * @param {string} [data.database] - 数据库名称 (可选，MySQL 可不填)
 * @param {string} [data.sqliteFile] - SQLite 文件路径 (SQLite 必填)
 * @param {string} [data.username] - 用户名 (可选)
 * @param {string} [data.password] - 密码 (可选)
 * @returns {Promise<string>} 连接测试结果消息
 */
export function testConnection(data) {
  const params = {
    type: data.type
  }

  // MySQL/PostgreSQL 特有字段
  if (data.type !== 'sqlite') {
    params.host = data.host || 'localhost'
    params.port = data.port || getDefaultPort(data.type)
    params.username = data.username || ''
    params.password = data.password || ''
    // MySQL 可选指定数据库，PostgreSQL 建议指定
    if (data.database) {
      params.database = data.database
    }
  }

  // SQLite 特有字段
  if (data.type === 'sqlite' && data.sqliteFile) {
    params.sqliteFile = data.sqliteFile
  }

  return invoke('test_datasource_connection', { params })
}

/**
 * 获取数据库类型的默认端口
 * @param {string} type - 数据库类型
 * @returns {number} 默认端口号
 */
export function getDefaultPort(type) {
  const portMap = {
    mysql: 3306,
    postgresql: 5432,
    sqlite: null
  }
  return portMap[type] || null
}

/**
 * 检查数据库类型是否需要网络连接配置
 * @param {string} type - 数据库类型
 * @returns {boolean} 是否需要网络配置
 */
export function requiresNetworkConfig(type) {
  return type !== 'sqlite'
}

/**
 * 获取数据库中的表列表
 * @param {Object} datasource - 数据源配置
 * @returns {Promise<Array>} 表列表
 */
export async function getDatabaseTables(datasource) {
  try {
    const result = await invoke('cmd_list_database_tables', {
      params: {
        type: datasource.type_,
        host: datasource.host,
        port: datasource.port,
        username: datasource.username,
        password: datasource.password,
        database: datasource.database,
        sqliteFile: datasource.sqlite_file
      }
    })
    return JSON.parse(result)
  } catch (error) {
    console.error('获取数据库表失败:', error)
    throw error
  }
}
