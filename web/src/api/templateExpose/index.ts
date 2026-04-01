import request from '@/utils/request';

/**
 * 变量数据缓存
 * key: templateId
 * value: { promise, data, timestamp }
 *
 * promise: 正在进行的请求 Promise（用于防止并发重复请求）
 * data: 已完成的响应数据
 * timestamp: 缓存时间戳
 */
const variablesDataCache = new Map();

/**
 * 缓存过期时间（5分钟）
 */
const CACHE_TTL = 5 * 60 * 1000;

/**
 * 清除指定模板的缓存
 * @param {number} templateId - 模板ID
 */
export const clearVariablesCache = (templateId) => {
  if (templateId) {
    variablesDataCache.delete(String(templateId));
    console.log(`✓ 已清除模板 ${templateId} 的变量数据缓存`);
  }
};

/**
 * 清除所有缓存
 */
export const clearAllVariablesCache = () => {
  variablesDataCache.clear();
  console.log('✓ 已清除所有变量数据缓存');
};

/**
 * 获取模板暴露字段（带缓存 + 防并发）
 * @param {Object} params - 参数对象
 * @param {number} params.templateId - 模板ID
 * @param {string} params.version - 版本号（可选）
 * @param {boolean} params.forceRefresh - 强制刷新，跳过缓存
 * @returns {Promise}
 */
export const getTemplateExpose = (params) => {
  const { templateId, forceRefresh = false } = params;
  const cacheKey = String(templateId);

  // 强制刷新：清除缓存
  if (forceRefresh) {
    console.log(`🔄 强制刷新: templateId=${templateId}`);
    variablesDataCache.delete(cacheKey);
  }

  // 检查缓存
  const cached = variablesDataCache.get(cacheKey);

  if (cached) {
    const now = Date.now();

    // 情况1：已有完成的响应数据且未过期
    if (cached.data && !cached.promise) {
      if (now - cached.timestamp < CACHE_TTL) {
        console.log(
          `✓ 使用缓存的变量数据: templateId=${templateId}, 已缓存 ${Math.round(
            (now - cached.timestamp) / 1000
          )}秒`
        );
        return Promise.resolve(cached.data);
      } else {
        console.log(`⚠️ 缓存已过期: templateId=${templateId}`);
        variablesDataCache.delete(cacheKey);
      }
    }

    // 情况2：有正在进行的请求（防止并发重复请求）
    if (cached.promise) {
      console.log(`⏳ 等待正在进行的请求: templateId=${templateId}`);
      return cached.promise;
    }
  }

  // 请求新数据
  console.log(`🌐 请求变量数据: templateId=${templateId}`);

  // 创建请求 Promise 并立即缓存（防止并发重复请求）
  const requestPromise = request({
    url: `/api/v1/editor/templates/${templateId}/variables/data`,
    method: 'GET',
  })
    .then((response) => {
      // 请求完成后，更新缓存：移除 promise，保存 data
      variablesDataCache.set(cacheKey, {
        promise: null,
        data: response,
        timestamp: Date.now(),
      });
      console.log(`✓ 变量数据已缓存: templateId=${templateId}`);
      return response;
    })
    .catch((error) => {
      // 请求失败时，清除缓存，允许重试
      variablesDataCache.delete(cacheKey);
      console.error(`❌ 请求失败，已清除缓存: templateId=${templateId}`, error);
      throw error;
    });

  // 立即缓存 Promise（在请求完成前，其他并发请求会使用这个 Promise）
  variablesDataCache.set(cacheKey, {
    promise: requestPromise,
    data: null,
    timestamp: null,
  });

  return requestPromise;
};

/**
 * 设置模板暴露字段（保存后清除缓存）
 * @param {Object} params - 参数对象
 * @param {number} params.templateId - 模板ID
 * @param {Object} params.varsSchema - 变量Schema定义
 * @param {string} params.version - 版本号（可选）
 * @returns {Promise}
 */
export const setTemplateExpose = (params) => {
  const { templateId, varsSchema, version } = params;

  return request({
    url: `/api/v1/editor/templates/${templateId}/variables/data`,
    method: 'POST',
    data: {
      content: JSON.stringify(varsSchema, null, 2), // 格式化输出，2个空格缩进
      version: version || '1.0',
    },
  }).then((response) => {
    // 保存成功后清除缓存
    clearVariablesCache(templateId);
    return response;
  });
};

/**
 * 获取模板测试数据
 * @param {Object} params - 参数对象
 * @param {number} params.templateId - 模板ID
 * @returns {Promise}
 */
export const getTemplateTestData = (params) => {
  const { templateId } = params;

  return request({
    url: `/api/v1/editor/templates/${templateId}/variables/test`,
    method: 'GET',
  }).then((response) => {
    // 后端返回格式: { code: 0, data: "...", message: "OK" }
    // response.data 是整个响应对象: { code: 0, data: "{\"username\":\"...\"}", message: "OK" }
    // 我们需要提取 response.data.data 字段（JSON字符串）并解析

    // response.data.data 是 JSON 字符串
    const jsonDataString = response.data?.data;

    if (!jsonDataString) {
      console.log('后端返回的 data 字段为空');
      return {};
    }

    if (typeof jsonDataString === 'string') {
      try {
        const parsed = JSON.parse(jsonDataString);
        console.log('成功解析测试数据:', parsed);
        return parsed;
      } catch (e) {
        console.error('解析测试数据 JSON 失败:', e, jsonDataString);
        return {};
      }
    }

    // 如果已经是对象，直接返回
    console.log('测试数据已经是对象:', jsonDataString);
    return jsonDataString;
  });
};

/**
 * 保存模板测试数据
 * @param {Object} params - 参数对象
 * @param {number} params.templateId - 模板ID
 * @param {Object} params.testData - 测试数据对象
 * @returns {Promise}
 */
export const setTemplateTestData = (params) => {
  const { templateId, testData } = params;

  return request({
    url: `/api/v1/editor/templates/${templateId}/variables/test`,
    method: 'POST',
    data: {
      content: JSON.stringify(testData, null, 2), // 格式化输出，2个空格缩进
    },
  });
};

/**
 * 删除模板暴露字段
 * @param {Object} params - 参数对象
 * @param {number} params.templateId - 模板ID
 * @param {string} params.version - 版本号（可选）
 * @returns {Promise}
 */
export const deleteTemplateExpose = (params) => {
  const { templateId, version } = params;
  const url = version
    ? `/api/v1/editor/templates/${templateId}/expose?version=${version}`
    : `/api/v1/editor/templates/${templateId}/expose`;

  return request({
    url,
    method: 'DELETE',
  });
};

/**
 * 获取模板暴露字段历史版本列表
 * @param {Object} params - 参数对象
 * @param {number} params.templateId - 模板ID
 * @returns {Promise}
 */
export const getTemplateExposeVersions = (params) => {
  const { templateId } = params;

  return request({
    url: `/api/v1/editor/templates/${templateId}/expose/versions`,
    method: 'GET',
  });
};
