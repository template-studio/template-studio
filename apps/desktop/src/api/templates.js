import request from '@/utils/request'

// 获取分类列表
export function getCategories(params = {}) {
  return request({
    url: '/api/v1/studio/categories',
    method: 'get',
    params: {
      all: params.all || 0,
      name: params.name || '',
    },
  })
}

// 获取语言列表
export function getLanguages(params = {}) {
  return request({
    url: '/api/v1/studio/languages',
    method: 'get',
    params: {
      all: params.all || 0,
      name: params.name || '',
      displayName: params.displayName || '',
      code: params.code || '',
      isPopular: params.isPopular,
    },
  })
}

// 获取模板列表
export function getTemplates(params = {}) {
  return request({
    url: '/api/v1/studio/templates/list',
    method: 'get',
    params,
  })
}

// 获取模板详情
export function getTemplateDetail(templateId) {
  return request({
    url: `/api/v1/studio/templates/${templateId}`,
    method: 'get',
  })
}

// 获取模板变量定义
export function getTemplateVariables(templateId) {
  return request({
    url: `/api/v1/studio/templates/${templateId}/variables`,
    method: 'get',
  })
}

// 获取模板文件列表
export function getTemplateFiles(templateId) {
  return request({
    url: `/api/v1/studio/templates/${templateId}/files`,
    method: 'get',
  })
}
