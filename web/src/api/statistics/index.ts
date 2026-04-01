import request from '@/utils/request';

// 获取统计总览数据
export function getOverview() {
  return request({
    url: '/api/v1/admin/statistics/overview',
    method: 'get',
  });
}

// 获取分类分布数据
export function getCategoryDistribution() {
  return request({
    url: '/api/v1/admin/statistics/category-distribution',
    method: 'get',
  });
}

// 获取语言流行度数据
export function getLanguagePopularity() {
  return request({
    url: '/api/v1/admin/statistics/language-popularity',
    method: 'get',
  });
}

// 获取模板复杂度数据
export function getTemplateComplexity() {
  return request({
    url: '/api/v1/admin/statistics/template-complexity',
    method: 'get',
  });
}

// 获取使用趋势数据
export function getUsageTrends(days = 30) {
  return request({
    url: '/api/v1/admin/statistics/usage-trends',
    method: 'get',
    params: { days },
  });
}
