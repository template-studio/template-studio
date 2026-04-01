import request from '@/utils/request';

// 新增模板文件
export function addTemplateFile(data) {
  return request({
    url: '/api/v1/editor/templateFiles/add',
    method: 'post',
    data,
  });
}

// 编辑模板文件
export function editTemplateFile(data) {
  return request({
    url: '/api/v1/editor/templateFiles/edit',
    method: 'put',
    data: {
      templateId: data.templateId,
      filePath: data.filePath,
      content: data.content,
    },
  });
}

// 重命名模板文件
export function renameTemplateFile(data) {
  return request({
    url: '/api/v1/editor/templateFiles/rename',
    method: 'put',
    data,
  });
}

// 移动模板文件
export function moveTemplateFile(data) {
  return request({
    url: '/api/v1/editor/templateFiles/move',
    method: 'put',
    data: {
      templateId: data.templateId,
      filePath: data.filePath,
      newPath: data.newPath,
    },
  });
}

// 删除模板文件
export function delTemplateFile(data) {
  return request({
    url: '/api/v1/editor/templateFiles/del',
    method: 'delete',
    params: data,
  });
}

// 批量删除模板文件
export function batchDelTemplateFile(idList) {
  return request({
    url: '/api/v1/editor/templateFiles/batchdel',
    method: 'delete',
    params: { id: idList },
  });
}

// 获取模板文件详情
export function getTemplateFileDetail(id) {
  return request({
    url: '/api/v1/editor/templateFiles/detail',
    method: 'get',
    params: { id },
  });
}

// 获取模板文件列表
export function listTemplateFiles(params) {
  return request({
    url: '/api/v1/editor/templateFiles/list',
    method: 'get',
    params,
  });
}

// 获取模板文件树
export function getTemplateFileTree(templateId) {
  return request({
    url: '/api/v1/editor/templateFiles/fileTree',
    method: 'get',
    params: { templateId },
  });
}

// 获取模板文件树
export function getTemplateFileContent(templateId, filePath) {
  return request({
    url: '/api/v1/editor/templateFiles/content',
    method: 'get',
    params: { templateId, filePath },
  });
}

// 上传ZIP包
export function uploadZipFile(templateId, file) {
  const formData = new FormData();
  formData.append('zipFile', file);
  formData.append('templateId', templateId);

  return request({
    url: '/api/v1/editor/templateFiles/uploadZip',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
}

// 上传文件
export function uploadCodeFile(templateId, file, parentPath) {
  const formData = new FormData();
  formData.append('file', file);
  formData.append('templateId', templateId);
  if (parentPath) formData.append('parentPath', parentPath);
  return request({
    url: '/api/v1/editor/templateFiles/uploadCode',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
}

// 上传文件
export function uploadCode(data) {
  return request({
    url: '/api/v1/editor/templateFiles/uploadCode',
    method: 'post',
    data,
  });
}

// 渲染模板文件
export function renderTemplate(data) {
  return request({
    url: '/api/v1/editor/templateFiles/render',
    method: 'post',
    data,
  });
}

// 渲染文件树
export function renderFileTree(data) {
  return request({
    url: '/api/v1/editor/templateFiles/renderFileTree',
    method: 'post',
    data,
  });
}

// 下载ZIP包
export function downloadZip(data) {
  return request({
    url: '/api/v1/editor/templateFiles/downloadZip',
    method: 'post',
    data,
    responseType: 'blob', // 重要：设置响应类型为blob
  });
}

// 设置文件生成条件
export function setFileCondition(data) {
  return request({
    url: '/api/v1/editor/templateFiles/setCondition',
    method: 'put',
    data,
  });
}

// 获取文件生成条件
export function getFileCondition(id) {
  return request({
    url: '/api/v1/editor/templateFiles/getCondition',
    method: 'get',
    params: { id },
  });
}

/**
 * 预览模板文件（编辑器预览，从工作目录读取）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {string} data.filePath - 文件路径
 * @param {Object} data.variables - 变量数据
 * @returns {Promise}
 */
export function previewTemplate(data) {
  return request({
    url: '/api/v1/template-files/preview',
    method: 'post',
    data,
  });
}

/**
 * 预览整个文件树（编辑器预览模式，从工作目录读取）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {Object} data.variables - 变量数据
 * @returns {Promise}
 */
export function previewFileTree(data) {
  return request({
    url: '/api/v1/template-files/preview-tree',
    method: 'post',
    data,
  });
}

/**
 * 生成模板文件（生产模式，从发布版本读取）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {string} data.filePath - 文件路径
 * @param {Object} data.variables - 变量数据
 * @returns {Promise}
 */
export function generateTemplate(data) {
  return request({
    url: '/api/v1/template-files/generate',
    method: 'post',
    data,
  });
}

/**
 * 生成整个文件树（生产模式，从发布版本读取）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {Object} data.variables - 变量数据
 * @param {string} data.version - 指定版本号（可选，不填则使用最新版本）
 * @returns {Promise}
 */
export function generateFileTree(data) {
  return request({
    url: '/api/v1/template-files/generate-tree',
    method: 'post',
    data,
  });
}

/**
 * 生成并下载ZIP文件（用户模式，从发布版本读取）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {Object} data.variables - 变量数据
 * @param {string} data.version - 指定版本号（可选，不填则使用最新版本）
 * @param {string} data.fileName - 文件名（可选）
 * @returns {Promise}
 */
export function generateZip(data) {
  return request({
    url: '/api/v1/template-files/generate-zip',
    method: 'post',
    data,
    responseType: 'blob', // 重要：设置响应类型为blob
  });
}

/**
 * 获取模板变量定义（用户模式，从发布版本读取）
 * @param {Object} params
 * @param {number} params.templateId - 模板ID
 * @param {string} params.version - 指定版本号（可选，不填则使用最新版本）
 * @returns {Promise}
 */
export function getTemplateVariables(params) {
  return request({
    url: '/api/v1/template-files/variables',
    method: 'get',
    params,
  });
}

/**
 * 清除模板渲染缓存
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @returns {Promise}
 */
export function clearRenderCache(data) {
  return request({
    url: '/api/v1/template-files/clear-cache',
    method: 'post',
    data,
  });
}

/**
 * 还原文件到上次提交状态（git restore）
 * @param {Object} data
 * @param {number} data.templateId - 模板ID
 * @param {string} data.filePath - 文件路径
 * @returns {Promise}
 */
export function restoreTemplateFile(data) {
  return request({
    url: '/api/v1/editor/templateFiles/restore',
    method: 'post',
    data,
  });
}
