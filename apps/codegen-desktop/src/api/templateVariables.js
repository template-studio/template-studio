import request from '@/utils/request'

/**
 * 获取模板的变量定义
 * @param {number} templateId - 模板ID
 * @param {string} version - 版本号
 * @returns {Promise}
 */
export function getTemplateVariables(templateId, version) {
  return request({
    url: '/api/v1/template-files/variables',
    method: 'get',
    params: {
      templateId,
      version
    }
  })
}
