import request from '@/utils/request';

/**
 * 订阅预设变量到模板
 * @param {number} templateId - 模板ID
 * @param {Array} presetIds - 预设变量ID列表
 * @returns {Promise}
 */
export function subscribePreset(templateId, presetIds) {
  return request({
    url: `/api/v1/editor/templates/${templateId}/preset-variables/subscribe`,
    method: 'POST',
    data: {
      template_id: templateId,
      preset_ids: presetIds,
    },
  });
}

/**
 * 获取模板订阅的预设变量列表
 * @param {number} templateId - 模板ID
 * @returns {Promise}
 */
export function getSubscribedPresets(templateId) {
  return request({
    url: `/api/v1/editor/templates/${templateId}/preset-variables`,
    method: 'GET',
  });
}

/**
 * 取消订阅预设变量
 * @param {number} templateId - 模板ID
 * @param {number} presetId - 预设变量ID
 * @returns {Promise}
 */
export function unsubscribePreset(templateId, presetId) {
  return request({
    url: `/api/v1/editor/templates/${templateId}/preset-variables/subscribe/${presetId}`,
    method: 'DELETE',
  });
}

/**
 * 获取可用预设变量列表
 * @param {Object} params - 查询参数
 * @returns {Promise}
 */
export function getAvailablePresets(params = {}) {
  return request({
    url: '/api/v1/editor/templates/preset-variables/available',
    method: 'GET',
    params: {
      pageNum: 1,
      pageSize: 20,
      ...params,
    },
  });
}
