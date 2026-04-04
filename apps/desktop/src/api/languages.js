/**
 * 语言 API 模块
 *
 * 提供语言的 CRUD 操作和项目语言关联功能
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 获取所有语言
 * @returns {Promise<Array>} 语言列表
 */
export function getAllLanguages() {
  return invoke('db_get_all_languages').then(data => JSON.parse(data))
}

/**
 * 根据 ID 获取语言
 * @param {number|string} id - 语言 ID
 * @returns {Promise<Object>} 语言对象
 */
export function getLanguage(id) {
  // 确保转换为数字类型
  const numId = typeof id === 'string' ? parseInt(id, 10) : id
  return invoke('db_get_language', { id: numId }).then(data => JSON.parse(data))
}

/**
 * 创建语言
 * @param {Object} data - 语言配置
 * @param {string} data.name - 语言名称
 * @param {string} [data.icon] - 图标（emoji）
 * @param {string} [data.color] - 颜色
 * @param {string} [data.description] - 描述
 * @returns {Promise<number>} 新创建的语言 ID
 */
export function createLanguage(data) {
  const params = {
    name: data.name,
    icon: data.icon || '',
    color: data.color || 'default',
    description: data.description || ''
  }

  return invoke('db_create_language', { params })
}

/**
 * 更新语言
 * @param {number|string} id - 语言 ID
 * @param {Object} data - 更新的语言配置
 * @param {string} data.name - 语言名称
 * @param {string} [data.icon] - 图标（emoji）
 * @param {string} [data.color] - 颜色
 * @param {string} [data.description] - 描述
 * @returns {Promise<void>}
 */
export function updateLanguage(id, data) {
  // 确保转换为数字类型
  const numId = typeof id === 'string' ? parseInt(id, 10) : id

  const params = {
    name: data.name,
    icon: data.icon || '',
    color: data.color || 'default',
    description: data.description || ''
  }

  return invoke('db_update_language', { id: numId, params })
}

/**
 * 删除语言
 * @param {number|string} id - 语言 ID
 * @returns {Promise<void>}
 */
export function deleteLanguage(id) {
  // 确保转换为数字类型
  const numId = typeof id === 'string' ? parseInt(id, 10) : id
  return invoke('db_delete_language', { id: numId })
}

/**
 * 设置项目的主语言
 * @param {number|string} projectId - 项目 ID
 * @param {number|string} languageId - 语言 ID
 * @returns {Promise<void>}
 */
export function setProjectPrimaryLanguage(projectId, languageId) {
  // 确保转换为数字类型
  const numProjectId = typeof projectId === 'string' ? parseInt(projectId, 10) : projectId
  const numLanguageId = typeof languageId === 'string' ? parseInt(languageId, 10) : languageId

  return invoke('db_set_project_primary_language', {
    projectId: numProjectId,
    languageId: numLanguageId
  })
}

/**
 * 获取项目的所有语言
 * @param {number|string} projectId - 项目 ID
 * @returns {Promise<Array>} 语言列表
 */
export function getProjectLanguages(projectId) {
  // 确保转换为数字类型
  const numId = typeof projectId === 'string' ? parseInt(projectId, 10) : projectId
  return invoke('db_get_project_languages', { projectId: numId }).then(data => JSON.parse(data))
}

/**
 * 为项目添加语言
 * @param {number|string} projectId - 项目 ID
 * @param {number|string} languageId - 语言 ID
 * @returns {Promise<void>}
 */
export function addProjectLanguage(projectId, languageId) {
  // 确保转换为数字类型
  const numProjectId = typeof projectId === 'string' ? parseInt(projectId, 10) : projectId
  const numLanguageId = typeof languageId === 'string' ? parseInt(languageId, 10) : languageId

  return invoke('db_add_project_language', {
    projectId: numProjectId,
    languageId: numLanguageId
  })
}

/**
 * 移除项目的语言
 * @param {number|string} projectId - 项目 ID
 * @param {number|string} languageId - 语言 ID
 * @returns {Promise<void>}
 */
export function removeProjectLanguage(projectId, languageId) {
  // 确保转换为数字类型
  const numProjectId = typeof projectId === 'string' ? parseInt(projectId, 10) : projectId
  const numLanguageId = typeof languageId === 'string' ? parseInt(languageId, 10) : languageId

  return invoke('db_remove_project_language', {
    projectId: numProjectId,
    languageId: numLanguageId
  })
}

/**
 * 获取语言的所有类型字段
 * @param {number|string} languageId - 语言 ID
 * @returns {Promise<Array>} 类型字段列表
 */
export function getLanguageFieldTypes(languageId) {
  const numId = typeof languageId === 'string' ? parseInt(languageId, 10) : languageId
  return invoke('db_get_language_field_types', { languageId: numId }).then(data => JSON.parse(data))
}

/**
 * 创建语言类型字段
 * @param {number|string} languageId - 语言 ID
 * @param {Object} data - 类型字段数据
 * @param {string} data.name - 类型名称
 * @param {string} [data.description] - 描述
 * @param {number} [data.sortOrder] - 排序顺序
 * @returns {Promise<number>} 新创建的字段 ID
 */
export function createLanguageFieldType(languageId, data) {
  const numId = typeof languageId === 'string' ? parseInt(languageId, 10) : languageId
  return invoke('db_create_language_field_type', {
    languageId: numId,
    name: data.name,
    description: data.description || '',
    sortOrder: data.sortOrder || 0
  })
}

/**
 * 更新语言类型字段
 * @param {number|string} id - 字段 ID
 * @param {Object} data - 类型字段数据
 * @param {string} data.name - 类型名称
 * @param {string} [data.description] - 描述
 * @param {number} [data.sortOrder] - 排序顺序
 * @returns {Promise<void>}
 */
export function updateLanguageFieldType(id, data) {
  const numId = typeof id === 'string' ? parseInt(id, 10) : id
  return invoke('db_update_language_field_type', {
    id: numId,
    name: data.name,
    description: data.description || '',
    sortOrder: data.sortOrder || 0
  })
}

/**
 * 删除语言类型字段
 * @param {number|string} id - 字段 ID
 * @returns {Promise<void>}
 */
export function deleteLanguageFieldType(id) {
  const numId = typeof id === 'string' ? parseInt(id, 10) : id
  return invoke('db_delete_language_field_type', { id: numId })
}

/**
 * 批量保存语言类型字段
 * @param {number|string} languageId - 语言 ID
 * @param {Array} fieldTypes - 类型字段数组
 * @returns {Promise<void>}
 */
export function batchSaveLanguageFieldTypes(languageId, fieldTypes) {
  const numId = typeof languageId === 'string' ? parseInt(languageId, 10) : languageId
  return invoke('db_batch_save_language_field_types', {
    languageId: numId,
    fieldTypes: JSON.stringify(fieldTypes)
  })
}
