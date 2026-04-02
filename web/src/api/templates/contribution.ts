import { Alova } from '@/utils/http/alova/index';

// ===== 用户模板 API =====

export function listMyTemplates(params) {
  return Alova.Get('/v1/admin/my/templates/list', { params, cacheFor: 0 });
}

export function createUserTemplate(data) {
  return Alova.Post('/v1/admin/my/templates/add', data);
}

export function updateUserTemplate(id, data) {
  return Alova.Put(`/v1/admin/my/templates/${id}`, data);
}

export function deleteUserTemplate(id) {
  return Alova.Delete(`/v1/admin/my/templates/${id}`);
}

export function submitForReview(id) {
  return Alova.Post(`/v1/admin/my/templates/${id}/submit-review`);
}

// ===== 公开模板 API =====

export function listPublicTemplates(params) {
  return Alova.Get('/v1/templates/list', { params, cacheFor: 60 * 1000 });
}

// ===== 管理员审核 API =====

export function listPendingTemplates(params) {
  return Alova.Get('/v1/admin/templates/pending/list', { params, cacheFor: 0 });
}

export function reviewTemplate(data) {
  return Alova.Post('/v1/admin/templates/pending/review', data);
}
