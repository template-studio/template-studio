import request from '@/utils/request'

/**
 * 生成并预览文件树（用户模式）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {Object} data.variables - 模板变量
 * @param {string} data.version - 版本号（可选）
 * @returns {Promise}
 */
export function generateFileTree(data) {
  return request({
    url: '/api/v1/template-files/generate-tree',
    method: 'post',
    data
  })
}

/**
 * 生成并下载ZIP文件
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {Object} data.variables - 模板变量
 * @param {string} data.fileName - 项目名称
 * @param {string} data.version - 版本号（可选）
 * @returns {Promise}
 */
export function generateZip(data) {
  return request({
    url: '/api/v1/template-files/generate-zip',
    method: 'post',
    data,
    responseType: 'arraybuffer'
  })
}
