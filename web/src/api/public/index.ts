import request from '@/utils/request';

// 获取公开分类列表
export function getPublicCategories(params: Record<string, any> = {}) {
  return request({
    url: '/api/v1/studio/categories',
    method: 'get',
    params: {
      all: params.all || 0,
      name: params.name || '',
    },
  });
}

// 获取公开语言列表
export function getPublicLanguages(params: Record<string, any> = {}) {
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
  });
}

// 获取公开模板类型列表
export function getPublicTemplateTypes(params: Record<string, any> = {}) {
  return request({
    url: '/api/v1/studio/template-types',
    method: 'get',
    params,
  });
}

//获取公开模板列表
export function getPublicTemplates(params: Record<string, any> = {}) {
  return request({
    url: '/api/v1/studio/templates/list',
    method: 'get',
    params,
  });
}
