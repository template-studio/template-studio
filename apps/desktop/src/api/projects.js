/**
 * 项目 API 模块
 *
 * 提供项目的 CRUD 操作和表结构导入功能
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 获取所有项目（包含关联的数据源信息）
 * @returns {Promise<Array>} 项目列表，每个项目包含 datasource 对象
 */
export function getAllProjects() {
  return invoke('db_get_all_projects').then(data => JSON.parse(data))
}

/**
 * 根据 ID 获取单个项目
 * @param {number|string} id - 项目 ID
 * @returns {Promise<Object>} 项目对象（包含关联的数据源信息）
 */
export function getProject(id) {
  // 确保转换为数字类型
  const numId = typeof id === 'string' ? parseInt(id, 10) : id
  return invoke('db_get_project', { id: numId }).then(data => JSON.parse(data))
}

/**
 * 创建项目
 * @param {Object} data - 项目配置
 * @param {string} data.name - 项目名称
 * @param {string} [data.description] - 项目描述
 * @param {number} data.datasourceId - 数据源 ID
 * @param {string} data.databaseName - 数据库名称（MySQL/PostgreSQL）或 SQLite 文件名
 * @param {number} [data.primaryLanguageId] - 主语言 ID
 * @returns {Promise<number>} 新创建的项目 ID
 */
export function createProject(data) {
  const params = {
    name: data.name,
    description: data.description || '',
    datasourceId: data.datasourceId,
    databaseName: data.databaseName,
    primaryLanguageId: data.primaryLanguageId || null
  }

  return invoke('db_create_project', { params })
}

/**
 * 更新项目
 * @param {number|string} id - 项目 ID
 * @param {Object} data - 更新的项目配置
 * @param {string} data.name - 项目名称
 * @param {string} [data.description] - 项目描述
 * @param {number} data.datasourceId - 数据源 ID
 * @param {string} data.databaseName - 数据库名称
 * @param {number} [data.primaryLanguageId] - 主语言 ID
 * @returns {Promise<void>}
 */
export function updateProject(id, data) {
  // 确保转换为数字类型
  const numId = typeof id === 'string' ? parseInt(id, 10) : id

  const params = {
    name: data.name,
    description: data.description || '',
    datasourceId: data.datasourceId,
    databaseName: data.databaseName,
    primaryLanguageId: data.primaryLanguageId || null
  }

  return invoke('db_update_project', { id: numId, params })
}

/**
 * 删除项目
 * @param {number|string} id - 项目 ID
 * @returns {Promise<void>}
 */
export function deleteProject(id) {
  // 确保转换为数字类型
  const numId = typeof id === 'string' ? parseInt(id, 10) : id
  return invoke('db_delete_project', { id: numId })
}

/**
 * 获取项目的所有表
 * @param {number|string} projectId - 项目 ID
 * @returns {Promise<Array>} 表列表
 */
export function getProjectTables(projectId) {
  // 确保转换为数字类型
  const numId = typeof projectId === 'string' ? parseInt(projectId, 10) : projectId
  return invoke('db_get_project_tables', { projectId: numId }).then(data => JSON.parse(data))
}

/**
 * 获取表的所有列
 * @param {number} tableId - 表 ID
 * @returns {Promise<Array>} 列列表
 */
export function getTableColumns(tableId) {
  return invoke('db_get_table_columns', { tableId }).then(data => JSON.parse(data))
}

/**
 * 删除表
 * @param {number} tableId - 表 ID
 * @returns {Promise<void>}
 */
export function deleteTable(tableId) {
  return invoke('db_delete_table', { tableId })
}

/**
 * 更新表信息
 * @param {number} tableId - 表 ID
 * @param {Object} data - 表数据
 * @param {string} data.name - 表名
 * @param {string} [data.comment] - 表说明
 * @param {string} [data.engine] - 表引擎
 * @param {string} data.tableType - 表类型（table/view）
 * @returns {Promise<void>}
 */
export function updateTable(tableId, data) {
  return invoke('db_update_table', {
    tableId,
    name: data.name,
    comment: data.comment || null,
    engine: data.engine || null,
    tableType: data.tableType
  })
}

/**
 * 创建列
 * @param {Object} data - 列数据
 * @param {number} data.tableId - 表 ID
 * @param {string} data.name - 列名
 * @param {string} data.dataType - 数据类型
 * @param {number} [data.length] - 长度
 * @param {boolean} data.isNullable - 是否可空
 * @param {boolean} data.isPrimaryKey - 是否主键
 * @param {boolean} data.isUnique - 是否唯一
 * @param {string} [data.defaultValue] - 默认值
 * @param {string} [data.comment] - 说明
 * @param {number} data.ordinalPosition - 位置
 * @returns {Promise<number>} 新创建的列 ID
 */
export function createColumn(data) {
  return invoke('db_create_column', {
    tableId: data.tableId,
    name: data.name,
    dataType: data.dataType,
    length: data.length || null,
    isNullable: data.isNullable,
    isPrimaryKey: data.isPrimaryKey,
    isUnique: data.isUnique,
    defaultValue: data.defaultValue || null,
    comment: data.comment || null,
    ordinalPosition: data.ordinalPosition
  })
}

/**
 * 更新列信息
 * @param {number} columnId - 列 ID
 * @param {Object} data - 列数据
 * @param {string} data.name - 列名
 * @param {string} data.dataType - 数据类型
 * @param {number} [data.length] - 长度
 * @param {boolean} data.isNullable - 是否可空
 * @param {boolean} data.isPrimaryKey - 是否主键
 * @param {boolean} data.isUnique - 是否唯一
 * @param {string} [data.defaultValue] - 默认值
 * @param {string} [data.comment] - 说明
 * @returns {Promise<void>}
 */
export function updateColumn(columnId, data) {
  return invoke('db_update_column', {
    columnId,
    name: data.name,
    dataType: data.dataType,
    length: data.length || null,
    isNullable: data.isNullable,
    isPrimaryKey: data.isPrimaryKey,
    isUnique: data.isUnique,
    defaultValue: data.defaultValue || null,
    comment: data.comment || null
  })
}

/**
 * 删除列
 * @param {number} columnId - 列 ID
 * @returns {Promise<void>}
 */
export function deleteColumn(columnId) {
  return invoke('db_delete_column', { columnId })
}

/**
 * 解析SQL并创建表和字段
 * @param {number} projectId - 项目 ID
 * @param {string} sqlContent - SQL内容
 * @param {string} sqlDialect - SQL方言（mysql/postgresql/sqlite）
 * @returns {Promise<string>} 结果消息
 */
export function parseSqlAndCreate(projectId, sqlContent, sqlDialect) {
  return invoke('cmd_parse_sql_and_create', {
    projectId,
    sqlContent,
    sqlDialect
  })
}

/**
 * 从数据源导入表结构
 * @param {number} projectId - 项目 ID
 * @param {number} datasourceId - 数据源 ID
 * @param {string} databaseName - 数据库名称
 * @returns {Promise<string>} 导入结果消息
 */
export function importTablesFromDatasource(projectId, datasourceId, databaseName) {
  return invoke('cmd_import_tables_from_datasource', {
    projectId,
    datasourceId,
    databaseName
  })
}
