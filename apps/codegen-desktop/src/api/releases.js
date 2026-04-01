import request from '@/utils/request'

/**
 * 获取模板的所有版本列表
 * @param {number} templateId - 模板ID
 * @returns {Promise}
 */
export function listReleases(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/releases`,
    method: 'get',
  })
}
