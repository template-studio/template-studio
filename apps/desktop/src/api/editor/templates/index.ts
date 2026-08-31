import request from '@/utils/apiRequest';
import { useConfigStore } from '@/stores/config';

// 模板-新增
export function addTemplate(data) {
  return request({
    url: '/api/v1/template/templates/add',
    method: 'post',
    data,
  });
}

// 模板-批量删除
export function batchDeleteTemplate(params) {
  return request({
    url: '/api/v1/template/templates/batchdel',
    method: 'delete',
    params,
  });
}

// 模板-删除
export function deleteTemplate(params) {
  return request({
    url: '/api/v1/template/templates/del',
    method: 'delete',
    params,
  });
}

// 模板-详情
export function getTemplateDetail(params) {
  return request({
    url: '/api/v1/template/templates/detail',
    method: 'get',
    params,
  });
}

// 模板-修改
export function editTemplate(data) {
  return request({
    url: '/api/v1/template/templates/edit',
    method: 'put',
    data,
  });
}

// 模板-列表
export function listTemplates(params) {
  return request({
    url: '/api/v1/template/templateList',
    method: 'get',
    params,
  });
}

// 模板-变量列表
export function getTemplateVariables(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/variables`,
    method: 'get',
  });
}

// 模板-分析变量
export function analyzeTemplateVariables(templateId) {
  return request({
    url: `/api/v1/template/templates/${templateId}/analyze-variables`,
    method: 'post',
  });
}

// 获取模板类型列表
export function getTemplateTypes() {
  return request({
    url: '/api/v1/template/templates/types',
    method: 'get',
  });
}

// Fork模板
export function forkTemplate(data) {
  return request({
    url: '/api/v1/admin/my/templates/fork',
    method: 'post',
    data,
  });
}

// 切换推荐状态
export function toggleTemplateFeatured(data) {
  return request({
    url: '/api/v1/template/templates/toggle-featured',
    method: 'put',
    data,
  });
}

// 导出模板 - 直接下载文件（直链下载无法带请求头，token 经查询参数传递）
// 桌面端差异：页面与 API 不同源，需拼接服务端绝对地址，token 取自设置里的 API Token
export function exportTemplate(templateId, format = 'files', fileName = null) {
  const configStore = useConfigStore();
  const token = configStore.apiKey || '';
  const exportUrl = `${configStore.baseURL}/api/v1/template/templates/${templateId}/export?format=${format}&token=${token}`;
  // 创建一个临时链接来触发文件下载
  const link = document.createElement('a');
  link.href = exportUrl;
  link.download = fileName || `template_${templateId}.${format === 'json' ? 'json' : 'zip'}`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
