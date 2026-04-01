/**
 * 表配置 API 模块
 * 提供表生成配置的 CRUD 操作
 */

import { invoke } from '@tauri-apps/api/core'

// ==================== 表配置 ====================

/**
 * 获取表配置
 * @param {number} tableId - 表 ID
 * @returns {Promise<Object|null>} 表配置
 */
export function getTableConfig(tableId) {
  return invoke('get_table_gen_config', { tableId })
    .then(data => data ? JSON.parse(data) : null)
    .catch(() => null)
}

/**
 * 保存表配置
 * @param {number} tableId - 表 ID
 * @param {Object} config - 配置数据
 * @returns {Promise<number>} 配置 ID
 */
export function saveTableConfig(tableId, config) {
  return invoke('save_table_gen_config', {
    tableId,
    config: JSON.stringify(config)
  })
}

// ==================== 字段配置 ====================

/**
 * 获取表的所有字段配置
 * @param {number} tableId - 表 ID
 * @returns {Promise<Array>} 字段配置列表
 */
export function getColumnConfigs(tableId) {
  return invoke('get_column_gen_configs', { tableId })
    .then(data => JSON.parse(data))
    .catch(() => [])
}

/**
 * 保存字段配置（批量）
 * @param {number} tableId - 表 ID
 * @param {Array} columns - 字段配置列表
 * @returns {Promise<void>}
 */
export function saveColumnConfigs(tableId, columns) {
  return invoke('save_column_gen_configs', {
    tableId,
    columns: JSON.stringify(columns)
  })
}

/**
 * 保存单个字段配置
 * @param {number} columnId - 字段 ID
 * @param {Object} config - 字段配置
 * @returns {Promise<void>}
 */
export function saveColumnConfig(columnId, config) {
  return invoke('save_column_gen_config', {
    columnId,
    config: JSON.stringify(config)
  })
}

// ==================== 类型映射 ====================

/**
 * 获取模板组的类型映射
 * @param {string} templateGroup - 模板组名称
 * @returns {Promise<Array>} 类型映射列表
 */
export function getTypeMappings(templateGroup) {
  return invoke('get_type_mappings', { templateGroup })
    .then(data => JSON.parse(data))
    .catch(() => [])
}

/**
 * 根据数据库类型获取目标类型
 * @param {string} dbType - 数据库类型
 * @param {string} templateGroup - 模板组名称
 * @returns {Promise<string>} 目标类型
 */
export function getTargetType(dbType, templateGroup) {
  return invoke('get_target_type', { dbType, templateGroup })
}

// ==================== 模板组 ====================

/**
 * 获取所有模板组
 * @returns {Promise<Array>} 模板组列表
 */
export function getTemplateGroups() {
  return invoke('get_template_groups')
    .then(data => JSON.parse(data))
    .catch(() => [])
}

/**
 * 获取模板组的配置 Schema
 * @param {string} templateGroup - 模板组名称
 * @returns {Promise<Array>} 配置 Schema
 */
export function getTemplateSchema(templateGroup) {
  return invoke('get_template_schema', { templateGroup })
    .then(data => JSON.parse(data))
    .catch(() => [])
}

// ==================== 代码生成 ====================

/**
 * 预览生成的代码
 * @param {number} tableId - 表 ID
 * @returns {Promise<Object>} 预览结果
 */
export function previewCode(tableId) {
  return invoke('preview_gen_code', { tableId })
    .then(data => JSON.parse(data))
}

/**
 * 生成代码
 * @param {number} tableId - 表 ID
 * @param {Object} options - 生成选项
 * @returns {Promise<string>} 生成结果
 */
export function generateCode(tableId, options = {}) {
  return invoke('generate_code', {
    tableId,
    options: JSON.stringify(options)
  })
}

/**
 * 批量生成代码
 * @param {Array<number>} tableIds - 表 ID 列表
 * @param {Object} options - 生成选项
 * @returns {Promise<string>} 生成结果
 */
export function batchGenerateCode(tableIds, options = {}) {
  return invoke('batch_generate_code', {
    tableIds,
    options: JSON.stringify(options)
  })
}

export default {
  getTableConfig,
  saveTableConfig,
  getColumnConfigs,
  saveColumnConfigs,
  saveColumnConfig,
  getTypeMappings,
  getTargetType,
  getTemplateGroups,
  getTemplateSchema,
  previewCode,
  generateCode,
  batchGenerateCode
}
