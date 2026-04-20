import request from '@/utils/request';

/**
 * 创建发布版本
 * @param {number} templateId - 模板ID
 * @param {Object} data - 发布数据
 * @param {string} data.version - 版本号（可选，不填则自动生成）
 * @param {string} data.changelog - 发布日志
 * @param {string} data.message - Git提交信息
 * @returns {Promise}
 */
export function createRelease(templateId, data) {
  return request({
    url: `/api/v1/template/templates/${templateId}/releases`,
    method: 'post',
    data,
  });
}

/**
 * 获取模板的所有版本列表
 * @param {number} templateId - 模板ID
 * @returns {Promise}
 */
export function listReleases(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/releases`,
    method: 'get',
  });
}

/**
 * 回滚到指定版本
 * @param {number} templateId - 模板ID
 * @param {string} version - 目标版本号
 * @returns {Promise}
 */
export function rollbackVersion(templateId, version) {
  return request({
    url: `/api/v1/template/templates/${templateId}/releases/${version}/rollback`,
    method: 'post',
  });
}

/**
 * 标记版本为已弃用
 * @param {number} templateId - 模板ID
 * @param {string} version - 版本号
 * @returns {Promise}
 */
export function deprecateVersion(templateId, version) {
  return request({
    url: `/api/v1/template/templates/${templateId}/releases/${version}/deprecate`,
    method: 'post',
  });
}

/**
 * 重置到最新版本（类似 git restore .）
 * 删除工作目录中的所有更改，恢复到最新发布版本的状态
 * @param {number} templateId - 模板ID
 * @returns {Promise<{version: string, deletedFiles: number, restoredFiles: number, addedFiles: number}>}
 */
export function resetToLatest(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/releases/reset-to-latest`,
    method: 'post',
  });
}
