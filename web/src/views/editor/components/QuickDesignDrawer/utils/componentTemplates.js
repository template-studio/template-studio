/**
 * componentTemplates - 组件模板定义
 *
 * 定义所有可拖拽的组件类型及其默认 Schema 结构
 */

import { TextOutline, RadioButtonOn, GridOutline, ChevronDownOutline, ListOutline } from '@/icons/ionicons5'

/**
 * 基础组件类型定义
 */
export const COMPONENT_TYPES = {
  STRING: 'string',       // 文本框
  INTEGER: 'integer',     // 数字框
  NUMBER: 'number',       // 浮点数
  BOOLEAN: 'boolean',     // 开关
  ENUM: 'enum',          // 下拉框
  SECRET: 'secret',      // 密码框
  OBJECT: 'object',      // 对象（容器）
  ARRAY: 'array',        // 数组（简单数组）
  OBJECT_ARR: 'object_arr' // 对象数组（容器数组）
}

/**
 * 组件分类
 */
export const COMPONENT_CATEGORIES = [
  {
    id: 'basic',
    name: '基础组件',
    icon: TextOutline,
    components: [
      {
        type: COMPONENT_TYPES.STRING,
        name: '文本框',
        icon: TextOutline,
        description: '单行文本输入',
        color: '#1890ff'
      },
      {
        type: COMPONENT_TYPES.INTEGER,
        name: '数字框',
        icon: RadioButtonOn,
        description: '整数输入',
        color: '#52c41a'
      },
      {
        type: COMPONENT_TYPES.NUMBER,
        name: '浮点数',
        icon: RadioButtonOn,
        description: '小数输入',
        color: '#13c2c2'
      },
      {
        type: COMPONENT_TYPES.BOOLEAN,
        name: '开关',
        icon: RadioButtonOn,
        description: '布尔值切换',
        color: '#fa8c16'
      },
      {
        type: COMPONENT_TYPES.ENUM,
        name: '下拉框',
        icon: ChevronDownOutline,
        description: '枚举值选择',
        color: '#722ed1'
      },
      {
        type: COMPONENT_TYPES.SECRET,
        name: '密码框',
        icon: TextOutline,
        description: '密码输入',
        color: '#eb2f96'
      }
    ]
  },
  {
    id: 'complex',
    name: '复杂组件',
    icon: GridOutline,
    components: [
      {
        type: COMPONENT_TYPES.OBJECT,
        name: '对象',
        icon: GridOutline,
        description: '嵌套对象（包含多个字段）',
        color: '#eb2f96',
        isContainer: true,
        containerType: 'object'
      },
      {
        type: COMPONENT_TYPES.ARRAY,
        name: '数组',
        icon: ListOutline,
        description: '简单数组（字符串、数字等）',
        color: '#722ed1',
        isContainer: true,
        containerType: 'array'
      },
      {
        type: COMPONENT_TYPES.OBJECT_ARR,
        name: '对象数组',
        icon: GridOutline,
        description: '对象数组（包含多个对象的列表）',
        color: '#9254de',
        isContainer: true,
        containerType: 'object_arr'
      }
    ]
  }
]

/**
 * 生成默认 Schema 模板
 */
export function generateDefaultSchema(componentType, fieldName) {
  const schemas = {
    [COMPONENT_TYPES.STRING]: {
      type: 'string',
      title: '',
      description: '',
      required: false,
      default: '',
      placeholder: '',
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.INTEGER]: {
      type: 'integer',
      title: '',
      description: '',
      required: false,
      default: 0,
      minimum: null,
      maximum: null,
      placeholder: '',
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.NUMBER]: {
      type: 'number',
      title: '',
      description: '',
      required: false,
      default: 0.0,
      minimum: null,
      maximum: null,
      placeholder: '',
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.BOOLEAN]: {
      type: 'boolean',
      title: '',
      description: '',
      required: false,
      default: false,
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.ENUM]: {
      type: 'enum',
      title: '',
      description: '',
      required: false,
      default: '',
      enum: [],
      enumNames: [],
      placeholder: '请选择',
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.SECRET]: {
      type: 'secret',
      title: '',
      description: '',
      required: false,
      default: '',
      placeholder: '',
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.OBJECT]: {
      type: 'object',
      title: '',
      description: '',
      required: false,
      properties: {},
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.ARRAY]: {
      type: 'array',
      title: '',
      description: '',
      required: false,
      items: { type: 'string' }, // 数组元素类型（与高级模式一致）
      default: [],
      insertText: `{{ ${fieldName} }}`
    },

    [COMPONENT_TYPES.OBJECT_ARR]: {
      type: 'object_arr',
      title: '',
      description: '',
      required: false,
      items: { // 与高级模式一致：使用 items 而不是直接用 properties
        type: 'object',
        properties: {}
      },
      default: [],
      insertText: `{{ ${fieldName} }}`
    }
  }

  return schemas[componentType] || schemas[COMPONENT_TYPES.STRING]
}

/**
 * 生成唯一字段名
 * @param {string} componentType - 组件类型
 * @param {Object} existingFields - 已存在的字段名集合 {fieldName: true}
 * @returns {string} 字段名
 */
export function generateFieldName(componentType, existingFields = {}) {
  // 类型到字段名前缀的映射
  const typePrefixMap = {
    [COMPONENT_TYPES.STRING]: 'text',
    [COMPONENT_TYPES.INTEGER]: 'number',
    [COMPONENT_TYPES.NUMBER]: 'decimal',
    [COMPONENT_TYPES.BOOLEAN]: 'flag',
    [COMPONENT_TYPES.ENUM]: 'option',
    [COMPONENT_TYPES.SECRET]: 'password',
    [COMPONENT_TYPES.OBJECT]: 'config',
    [COMPONENT_TYPES.ARRAY]: 'list',
    [COMPONENT_TYPES.OBJECT_ARR]: 'items'
  }

  const prefix = typePrefixMap[componentType] || 'field'

  // 查找可用的字段名（避免重复）
  let counter = 1
  let fieldName
  do {
    fieldName = `${prefix}${counter}`
    counter++
  } while (existingFields[fieldName])

  return fieldName
}

/**
 * 生成组件的友好标题
 * @param {string} componentType - 组件类型
 * @param {number} index - 组件索引
 * @returns {string} 标题
 */
export function generateComponentTitle(componentType, index = 1) {
  const titleMap = {
    [COMPONENT_TYPES.STRING]: `文本字段${index}`,
    [COMPONENT_TYPES.INTEGER]: `数字字段${index}`,
    [COMPONENT_TYPES.NUMBER]: `浮点数字段${index}`,
    [COMPONENT_TYPES.BOOLEAN]: `布尔开关${index}`,
    [COMPONENT_TYPES.ENUM]: `枚举选择${index}`,
    [COMPONENT_TYPES.SECRET]: `密码字段${index}`,
    [COMPONENT_TYPES.OBJECT]: `配置对象${index}`,
    [COMPONENT_TYPES.ARRAY]: `数组${index}`,
    [COMPONENT_TYPES.OBJECT_ARR]: `对象列表${index}`
  }

  return titleMap[componentType] || `字段${index}`
}

/**
 * 智能类型推断
 * 根据字段名推断合适的组件类型
 */
export function inferComponentType(fieldName) {
  const name = fieldName.toLowerCase()

  // 对象类型推断（包含 config, settings, data, info 等关键词）
  if (/(config|settings|options|metadata|data|database|server|connection|profile|details|info)/.test(name)) {
    return COMPONENT_TYPES.OBJECT
  }

  // 数组类型推断（复数形式或包含 list, items 等关键词）
  if (/(items|elements|list|entries|rows|records|files|images|tags|categories|options|choices)/.test(name) || /[a-z]s$/.test(name)) {
    // 进一步判断是简单数组还是对象数组
    if (/(users|items|products|orders|files|records|rows|entries)/.test(name)) {
      return COMPONENT_TYPES.OBJECT_ARR
    }
    return COMPONENT_TYPES.ARRAY
  }

  // 整数类型推断
  if (/^(port|count|size|length|num|number|quantity|total|index|priority|order|level|degree|percent|percentage)/.test(name)) {
    return COMPONENT_TYPES.INTEGER
  }

  // 浮点数类型推断
  if (/^(price|cost|rate|ratio|score|weight|height|width|temperature|latitude|longitude|factor|coefficient)/.test(name)) {
    return COMPONENT_TYPES.NUMBER
  }

  // 布尔类型推断
  if (/^(is|has|can|should|will|enable|disable|active|disabled|visible|readonly|required|optional|allow|deny|check|uncheck)/.test(name)) {
    return COMPONENT_TYPES.BOOLEAN
  }

  // 枚举类型推断
  if (/^(type|kind|category|status|state|mode|role|level|grade|class|group|tag|label|environment|env|region|zone)/.test(name)) {
    return COMPONENT_TYPES.ENUM
  }

  // 密码类型推断
  if (/^(password|passwd|pwd|secret|token|key|apiKey|secretKey|auth)/.test(name)) {
    return COMPONENT_TYPES.SECRET
  }

  // 默认为字符串
  return COMPONENT_TYPES.STRING
}

/**
 * 生成默认标题（用户友好名称）
 */
export function generateDefaultTitle(fieldName) {
  // 移除常见前缀
  let title = fieldName.replace(/^(is|has|can|the|a|an)_/i, '')

  // 转换下划线为空格
  title = title.replace(/_/g, ' ')

  // 转换驼峰为空格
  title = title.replace(/([a-z])([A-Z])/g, '$1 $2')

  // 首字母大写
  title = title.charAt(0).toUpperCase() + title.slice(1)

  return title
}

/**
 * 验证 Schema 结构
 */
export function validateSchema(schema) {
  const errors = []

  if (!schema.type) {
    errors.push('缺少 type 字段')
  }

  if (!schema.title) {
    errors.push('缺少 title 字段')
  }

  // 特定类型的验证
  if (schema.type === COMPONENT_TYPES.ENUM) {
    if (!schema.enum || !Array.isArray(schema.enum) || schema.enum.length === 0) {
      errors.push('enum 类型必须有 enum 数组且不为空')
    }
  }

  if (schema.type === COMPONENT_TYPES.INTEGER || schema.type === COMPONENT_TYPES.NUMBER) {
    if (schema.minimum !== null && schema.maximum !== null) {
      if (schema.minimum >= schema.maximum) {
        errors.push('minimum 必须小于 maximum')
      }
    }
  }

  return {
    valid: errors.length === 0,
    errors
  }
}

/**
 * 获取组件显示信息
 */
export function getComponentDisplayInfo(componentType) {
  const info = {
    [COMPONENT_TYPES.STRING]: {
      label: '字符串',
      color: '#1890ff',
      bgColor: '#e6f7ff',
      borderColor: '#91d5ff'
    },
    [COMPONENT_TYPES.INTEGER]: {
      label: '整数',
      color: '#52c41a',
      bgColor: '#f6ffed',
      borderColor: '#b7eb8f'
    },
    [COMPONENT_TYPES.NUMBER]: {
      label: '数字',
      color: '#13c2c2',
      bgColor: '#e6fffb',
      borderColor: '#87e8de'
    },
    [COMPONENT_TYPES.BOOLEAN]: {
      label: '布尔',
      color: '#fa8c16',
      bgColor: '#fff7e6',
      borderColor: '#ffd591'
    },
    [COMPONENT_TYPES.ENUM]: {
      label: '枚举',
      color: '#722ed1',
      bgColor: '#f9f0ff',
      borderColor: '#d3adf7'
    },
    [COMPONENT_TYPES.SECRET]: {
      label: '密码',
      color: '#eb2f96',
      bgColor: '#fff0f6',
      borderColor: '#ffadd2'
    },
    [COMPONENT_TYPES.OBJECT]: {
      label: '对象',
      color: '#eb2f96',
      bgColor: '#fff0f6',
      borderColor: '#ffadd2'
    },
    [COMPONENT_TYPES.ARRAY]: {
      label: '数组',
      color: '#722ed1',
      bgColor: '#f9f0ff',
      borderColor: '#d3adf7'
    },
    [COMPONENT_TYPES.OBJECT_ARR]: {
      label: '对象数组',
      color: '#9254de',
      bgColor: '#f9f0ff',
      borderColor: '#d3adf7'
    }
  }

  return info[componentType] || info[COMPONENT_TYPES.STRING]
}

/**
 * Schema 转换工具函数
 */
export const SchemaConverter = {
  /**
   * 将组件列表转换为 Schema 对象
   */
  componentsToSchema(components) {
    const schema = {}
    components.forEach(component => {
      if (component.fieldName && component.schema) {
        schema[component.fieldName] = { ...component.schema }
      }
    })
    return schema
  },

  /**
   * 将 Schema 对象转换为组件列表
   */
  schemaToComponents(schema) {
    const components = []
    Object.entries(schema).forEach(([fieldName, fieldSchema]) => {
      components.push({
        id: `comp_${fieldName}_${Date.now()}`,
        fieldName,
        schema: { ...fieldSchema },
        type: fieldSchema.type
      })
    })
    return components
  }
}
