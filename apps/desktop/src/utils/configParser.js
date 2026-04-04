/**
 * 配置解析工具
 * 根据约定解析配置结构，决定渲染方式
 */

/**
 * 配置区块类型
 */
export const SectionType = {
  BASIC: 'basic',       // 基本信息（单个变量）
  FIELDS: 'fields',     // 字段配置（可编辑表格）
  OPTIONS: 'options',   // 生成选项（开关组）
  EXTRA: 'extra'        // 扩展配置（动态表单）
}

/**
 * 字段标签映射（中文）
 */
const FIELD_LABELS = {
  tableName: '表名',
  tableComment: '表注释',
  tableType: '表类型',
  entityName: '实体名',
  entityComment: '功能描述',
  businessName: '业务名',
  moduleName: '模块名',
  author: '作者',
  namingCase: '命名风格',
  tablePrefix: '表前缀',
  packageName: '包名'
}

/**
 * 只读字段（包含这些关键字的字段设为只读）
 */
const READONLY_PATTERNS = [
  /name$/i,
  /Name$/,
  /^table/,
  /^tableComment/,
  /^tableType/
]

/**
 * 判断字段是否只读
 */
export function isReadonlyField(key) {
  return READONLY_PATTERNS.some(pattern => pattern.test(key))
}

/**
 * 推断字段类型
 * @param {string} key 字段名
 * @param {any} value 字段值
 * @returns {string} 字段类型
 */
export function inferFieldType(key, value) {
  // 布尔值
  if (typeof value === 'boolean') {
    return 'switch'
  }

  // 枚举类型
  if (key === 'namingCase') {
    return 'select'
  }

  // 数字
  if (typeof value === 'number') {
    return 'number'
  }

  // 默认文本
  return 'text'
}

/**
 * 获取字段标签
 */
export function getFieldLabel(key) {
  return FIELD_LABELS[key] || key
}

/**
 * 获取字段的选项（用于下拉选择）
 */
export function getFieldOptions(key) {
  const OPTIONS_MAP = {
    namingCase: [
      { value: 'camelCase', label: 'camelCase' },
      { value: 'PascalCase', label: 'PascalCase' },
      { value: 'snake_case', label: 'snake_case' },
      { value: 'kebab-case', label: 'kebab-case' }
    ]
  }
  return OPTIONS_MAP[key] || null
}

/**
 * 解析配置，提取各个区块
 * @param {Object} config 配置对象
 * @returns {Array} 区块列表
 */
export function parseConfig(config) {
  if (!config || typeof config !== 'object') {
    return []
  }

  const sections = []

  // 1. 收集单个变量 → 基本信息区块
  const basicFields = []
  const specialKeys = ['fields', 'options', 'extra']

  Object.entries(config).forEach(([key, value]) => {
    if (specialKeys.includes(key)) return
    if (key.startsWith('_')) return

    // 只收集原始类型
    if (value === null || typeof value !== 'object') {
      basicFields.push({
        key,
        value,
        type: inferFieldType(key, value),
        label: getFieldLabel(key),
        readonly: isReadonlyField(key),
        options: getFieldOptions(key)
      })
    }
  })

  if (basicFields.length > 0) {
    sections.push({
      type: SectionType.BASIC,
      title: '基本信息',
      icon: 'setting',
      fields: basicFields
    })
  }

  // 2. fields → 字段配置区块
  if (config.fields && config.fields._data) {
    sections.push({
      type: SectionType.FIELDS,
      title: '字段配置',
      icon: 'table',
      columns: config.fields._columns || getDefaultFieldColumns(),
      data: config.fields._data
    })
  }

  // 3. options → 生成选项区块
  if (config.options && typeof config.options === 'object') {
    const optionItems = Object.entries(config.options)
      .filter(([key, value]) => typeof value === 'boolean')
      .map(([key, value]) => ({
        key,
        value,
        label: getFieldLabel(key)
      }))

    if (optionItems.length > 0) {
      sections.push({
        type: SectionType.OPTIONS,
        title: '生成选项',
        icon: 'check-circle',
        items: optionItems
      })
    }
  }

  // 4. extra → 扩展配置区块（有内容则显示）
  if (config.extra && typeof config.extra === 'object' && Object.keys(config.extra).length > 0) {
    const extraItems = Object.entries(config.extra)
      .filter(([key]) => !key.startsWith('_'))
      .map(([key, item]) => {
        // 解析 extra 中的配置项
        if (typeof item === 'object' && item._type) {
          return {
            key,
            type: item._type,
            label: item._label || key,
            options: item._options || null,
            value: item.value
          }
        }
        // 简单值
        return {
          key,
          type: inferFieldType(key, item),
          label: getFieldLabel(key),
          options: null,
          value: item
        }
      })

    if (extraItems.length > 0) {
      sections.push({
        type: SectionType.EXTRA,
        title: '扩展配置',
        icon: 'extension',
        items: extraItems
      })
    }
  }

  return sections
}

/**
 * 默认的字段表格列定义
 */
export function getDefaultFieldColumns() {
  return [
    { key: 'name', title: '字段名', width: 120, readonly: true },
    { key: 'field', title: '属性名', width: 100 },
    { key: 'type', title: '类型', width: 80 },
    { key: 'label', title: '显示名', width: 100 },
    {
      key: 'input',
      title: '输入类型',
      width: 90,
      type: 'select',
      options: [
        { value: 'text', label: '文本' },
        { value: 'textarea', label: '多行文本' },
        { value: 'number', label: '数字' },
        { value: 'select', label: '下拉' },
        { value: 'switch', label: '开关' },
        { value: 'date', label: '日期' },
        { value: 'datetime', label: '日期时间' },
        { value: 'hidden', label: '隐藏' }
      ]
    },
    { key: 'list', title: '列表', width: 50, type: 'switch' },
    { key: 'form', title: '表单', width: 50, type: 'switch' },
    { key: 'query', title: '查询', width: 50, type: 'switch' },
    { key: 'required', title: '必填', width: 50, type: 'switch' },
    { key: 'dict', title: '字典', width: 100 }
  ]
}

/**
 * 创建默认配置
 * @param {Object} table 表信息
 * @param {Array} columns 字段列表
 * @returns {Object} 默认配置
 */
export function createDefaultConfig(table, columns = []) {
  return {
    // 基本信息
    tableName: table?.name || '',
    tableComment: table?.comment || '',
    tableType: table?.table_type || 'table',
    entityName: '',
    entityComment: '',
    businessName: '',
    moduleName: '',
    author: '',
    namingCase: 'camelCase',
    tablePrefix: '',
    packageName: '',

    // 字段配置
    fields: {
      _columns: getDefaultFieldColumns(),
      _data: columns.map(col => ({
        name: col.name,
        field: '',
        type: '',
        label: col.comment || col.name,
        input: 'text',
        list: true,
        form: true,
        query: false,
        required: !col.is_nullable,
        dict: ''
      }))
    },

    // 生成选项
    options: {
      lombok: true,
      swagger: true,
      validation: true,
      serializable: true,
      logicDelete: true,
      autoFill: true,
      restful: true,
      permission: false
    },

    // 扩展配置（根据模板组动态填充）
    extra: {}
  }
}

export default {
  SectionType,
  parseConfig,
  createDefaultConfig,
  getDefaultFieldColumns,
  inferFieldType,
  getFieldLabel,
  isReadonlyField,
  getFieldOptions
}
