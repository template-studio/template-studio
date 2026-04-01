import request from '@/utils/request';

/**
 * 文件条件管理 API（方案B：使用 templateId + filePath）
 *
 * 条件保存在仓库的 .meta/variables/conditions.yml 文件中
 * 随模板一起版本控制和导出
 */

// 获取文件生成条件
// GET /api/v1/editor/file-conditions?templateId=:templateId&filePath=:filePath
export function getFileCondition(templateId, filePath) {
  return request({
    url: '/api/v1/editor/file-conditions',
    method: 'get',
    params: {
      templateId,
      filePath,
    },
  });
}

// 设置文件生成条件
// POST /api/v1/editor/file-conditions
export function setFileCondition(templateId, filePath, condition) {
  return request({
    url: '/api/v1/editor/file-conditions',
    method: 'post',
    data: {
      templateId: parseInt(templateId), // 转换为整数
      filePath,
      condition,
    },
  });
}

// 删除文件生成条件
// DELETE /api/v1/editor/file-conditions
export function deleteFileCondition(templateId, filePath) {
  return request({
    url: '/api/v1/editor/file-conditions',
    method: 'delete',
    data: {
      templateId: parseInt(templateId), // 转换为整数
      filePath,
    },
  });
}

// 评估文件条件（用于测试）
// POST /api/v1/editor/file-conditions/evaluate
export function evaluateFileCondition(templateId, filePath, variables) {
  return request({
    url: '/api/v1/editor/file-conditions/evaluate',
    method: 'post',
    data: {
      templateId: parseInt(templateId), // 转换为整数
      filePath,
      variables,
    },
  });
}

// 导出模板的条件为 YAML
// GET /api/v1/editor/templates/:templateId/conditions/export
export function exportConditionsYaml(templateId) {
  return request({
    url: `/api/v1/editor/templates/${templateId}/conditions/export`,
    method: 'get',
  });
}

// 从 YAML 导入条件
// POST /api/v1/editor/templates/:templateId/conditions/import
export function importConditionsYaml(templateId, yamlContent) {
  return request({
    url: `/api/v1/editor/templates/${templateId}/conditions/import`,
    method: 'post',
    data: {
      yaml: yamlContent,
    },
  });
}

/**
 * 条件类型常量
 */
export const ConditionTypes = {
  IF: 'if', // 单一条件
  AND: 'and', // 所有条件都为真
  OR: 'or', // 任一条件为真
  NOT: 'not', // 条件为假
  SWITCH: 'switch', // 多分支条件
};

/**
 * 操作符常量
 */
export const Operators = {
  EQ: 'eq', // 等于
  NE: 'ne', // 不等于
  GT: 'gt', // 大于
  LT: 'lt', // 小于
  GTE: 'gte', // 大于等于
  LTE: 'lte', // 小于等于
  IN: 'in', // 包含于
  NOT_IN: 'not_in', // 不包含于
  CONTAINS: 'contains', // 包含字符串
};

/**
 * 操作符显示名称映射
 */
export const OperatorLabels = {
  [Operators.EQ]: '等于',
  [Operators.NE]: '不等于',
  [Operators.GT]: '大于',
  [Operators.LT]: '小于',
  [Operators.GTE]: '大于等于',
  [Operators.LTE]: '小于等于',
  [Operators.IN]: '包含于',
  [Operators.NOT_IN]: '不包含于',
  [Operators.CONTAINS]: '包含字符串',
};

/**
 * 条件类型显示名称映射
 */
export const ConditionTypeLabels = {
  [ConditionTypes.IF]: '条件',
  [ConditionTypes.AND]: '且（全部满足）',
  [ConditionTypes.OR]: '或（任一满足）',
  [ConditionTypes.NOT]: '非（取反）',
  [ConditionTypes.SWITCH]: '多分支',
};
