/**
 * 命名转换工具
 * 支持多种命名风格的互相转换
 */

/**
 * 命名风格类型
 */
export const NamingCase = {
  CAMEL_CASE: 'camelCase',      // userName
  PASCAL_CASE: 'PascalCase',    // UserName
  SNAKE_CASE: 'snake_case',     // user_name
  KEBAB_CASE: 'kebab-case',     // user-name
  UPPER_SNAKE: 'UPPER_SNAKE'    // USER_NAME
}

/**
 * 命名转换器
 */
export const NamingConverter = {
  /**
   * 将字符串拆分为单词数组
   * @param {string} str 原始字符串
   * @returns {string[]} 单词数组
   */
  toWords(str) {
    if (!str) return []

    // 处理 snake_case
    if (str.includes('_')) {
      return str.split('_').filter(Boolean)
    }

    // 处理 kebab-case
    if (str.includes('-')) {
      return str.split('-').filter(Boolean)
    }

    // 处理 camelCase / PascalCase
    // 在大写字母前添加空格，然后分割
    return str
      .replace(/([A-Z])/g, ' $1')
      .trim()
      .split(/\s+/)
      .filter(Boolean)
  },

  /**
   * 首字母大写
   * @param {string} str 字符串
   * @returns {string}
   */
  capitalize(str) {
    if (!str) return ''
    return str.charAt(0).toUpperCase() + str.slice(1).toLowerCase()
  },

  /**
   * 转换为目标命名风格
   * @param {string} str 原始字符串
   * @param {string} targetCase 目标命名风格
   * @returns {string}
   */
  convert(str, targetCase) {
    if (!str) return ''

    const words = this.toWords(str)
    if (words.length === 0) return str

    switch (targetCase) {
      case NamingCase.CAMEL_CASE:
        return words.map((w, i) =>
          i === 0 ? w.toLowerCase() : this.capitalize(w)
        ).join('')

      case NamingCase.PASCAL_CASE:
        return words.map(w => this.capitalize(w)).join('')

      case NamingCase.SNAKE_CASE:
        return words.map(w => w.toLowerCase()).join('_')

      case NamingCase.UPPER_SNAKE:
        return words.map(w => w.toUpperCase()).join('_')

      case NamingCase.KEBAB_CASE:
        return words.map(w => w.toLowerCase()).join('-')

      default:
        return str
    }
  },

  /**
   * 去除表前缀
   * @param {string} str 原始字符串
   * @param {string} prefix 前缀
   * @returns {string}
   */
  removePrefix(str, prefix) {
    if (!prefix || !str) return str
    const regex = new RegExp(`^${prefix}`, 'i')
    return str.replace(regex, '')
  },

  /**
   * 获取所有命名风格的转换结果
   * @param {string} str 原始字符串
   * @param {string} prefix 表前缀（可选）
   * @returns {Object} 各种命名风格的结果
   */
  getAllCases(str, prefix = '') {
    const cleaned = this.removePrefix(str, prefix)
    return {
      original: str,
      cleaned,
      camelCase: this.convert(cleaned, NamingCase.CAMEL_CASE),
      pascalCase: this.convert(cleaned, NamingCase.PASCAL_CASE),
      snakeCase: this.convert(cleaned, NamingCase.SNAKE_CASE),
      kebabCase: this.convert(cleaned, NamingCase.KEBAB_CASE),
      upperSnake: this.convert(cleaned, NamingCase.UPPER_SNAKE)
    }
  }
}

/**
 * 数据库类型到通用类型的映射
 * 用于推断输入类型
 */
export const DbTypeToInputType = {
  // 文本类型
  'VARCHAR': 'text',
  'CHAR': 'text',
  'TEXT': 'textarea',
  'LONGTEXT': 'textarea',
  'TINYTEXT': 'text',
  'MEDIUMTEXT': 'textarea',

  // 数字类型
  'INT': 'number',
  'INTEGER': 'number',
  'TINYINT': 'number',
  'SMALLINT': 'number',
  'MEDIUMINT': 'number',
  'BIGINT': 'number',
  'FLOAT': 'number',
  'DOUBLE': 'number',
  'DECIMAL': 'number',
  'NUMERIC': 'number',

  // 日期时间类型
  'DATE': 'date',
  'DATETIME': 'datetime',
  'TIMESTAMP': 'datetime',
  'TIME': 'time',
  'YEAR': 'number',

  // 布尔类型
  'BOOLEAN': 'switch',
  'BIT': 'switch',

  // 其他类型
  'JSON': 'textarea',
  'BLOB': 'file',
  'UUID': 'text'
}

/**
 * 根据数据库类型推断输入类型
 * @param {string} dbType 数据库类型
 * @returns {string} 输入类型
 */
export function guessInputType(dbType) {
  if (!dbType) return 'text'

  // 提取基础类型（去除长度等参数）
  const baseType = dbType.replace(/\([^)]*\)/, '').toUpperCase()

  return DbTypeToInputType[baseType] || 'text'
}

/**
 * 查询类型
 */
export const QueryType = {
  EQ: 'eq',           // 等于
  NE: 'ne',           // 不等于
  LIKE: 'like',       // 模糊匹配
  LIKE_LEFT: 'likeLeft',   // 左模糊
  LIKE_RIGHT: 'likeRight', // 右模糊
  IN: 'in',           // 包含
  NOT_IN: 'notIn',    // 不包含
  BETWEEN: 'between', // 范围
  GT: 'gt',           // 大于
  GTE: 'gte',         // 大于等于
  LT: 'lt',           // 小于
  LTE: 'lte',         // 小于等于
  IS_NULL: 'isNull',  // 为空
  IS_NOT_NULL: 'isNotNull' // 不为空
}

/**
 * 查询类型选项
 */
export const QueryTypeOptions = [
  { value: 'eq', label: '等于' },
  { value: 'ne', label: '不等于' },
  { value: 'like', label: '模糊' },
  { value: 'likeLeft', label: '左模糊' },
  { value: 'likeRight', label: '右模糊' },
  { value: 'in', label: '包含' },
  { value: 'between', label: '范围' },
  { value: 'gt', label: '大于' },
  { value: 'gte', label: '大于等于' },
  { value: 'lt', label: '小于' },
  { value: 'lte', label: '小于等于' }
]

/**
 * 输入类型选项
 */
export const InputTypeOptions = [
  { value: 'text', label: '文本输入' },
  { value: 'textarea', label: '多行文本' },
  { value: 'number', label: '数字输入' },
  { value: 'select', label: '下拉选择' },
  { value: 'radio', label: '单选框' },
  { value: 'checkbox', label: '复选框' },
  { value: 'switch', label: '开关' },
  { value: 'date', label: '日期' },
  { value: 'datetime', label: '日期时间' },
  { value: 'time', label: '时间' },
  { value: 'file', label: '文件上传' },
  { value: 'image', label: '图片上传' },
  { value: 'editor', label: '富文本' },
  { value: 'password', label: '密码框' },
  { value: 'color', label: '颜色选择' },
  { value: 'custom', label: '自定义组件' }
]

/**
 * 命名风格选项
 */
export const NamingCaseOptions = [
  { value: 'camelCase', label: 'camelCase (userName)', description: '首字母小写的驼峰命名' },
  { value: 'PascalCase', label: 'PascalCase (UserName)', description: '首字母大写的驼峰命名' },
  { value: 'snake_case', label: 'snake_case (user_name)', description: '下划线连接的小写命名' },
  { value: 'kebab-case', label: 'kebab-case (user-name)', description: '短横线连接的小写命名' }
]

export default {
  NamingCase,
  NamingConverter,
  DbTypeToInputType,
  guessInputType,
  QueryType,
  QueryTypeOptions,
  InputTypeOptions,
  NamingCaseOptions
}
