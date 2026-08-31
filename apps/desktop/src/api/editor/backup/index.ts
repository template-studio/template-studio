/**
 * 备份恢复 API
 * 调用后端 Rust 实现的备份恢复接口
 */

import request from '@/utils/apiRequest';

/**
 * 创建模板备份
 * @param params
 * @param params.templateId - 模板 ID
 * @param params.includeTestData - 是否包含测试数据
 * @param params.includeConditions - 是否包含文件条件
 * @returns Promise<Blob> - 返回 .tsbk 备份文件
 */
export function createBackup(params: {
  templateId: number;
  includeTestData?: boolean;
  includeConditions?: boolean;
}): Promise<Blob> {
  const queryParams = new URLSearchParams();
  queryParams.append('templateId', String(params.templateId));
  if (params.includeTestData !== undefined) {
    queryParams.append('includeTestData', String(params.includeTestData));
  }
  if (params.includeConditions !== undefined) {
    queryParams.append('includeConditions', String(params.includeConditions));
  }

  return request({
    url: `/api/v1/backup/create?${queryParams.toString()}`,
    method: 'get',
    responseType: 'blob',
  });
}

/**
 * 预览备份文件
 * @param file - 备份文件 (.tsbk)
 * @returns Promise<BackupPreviewResponse>
 */
export function previewBackup(file: File): Promise<BackupPreviewResponse> {
  const formData = new FormData();
  formData.append('backupFile', file);

  return request({
    url: '/api/v1/backup/preview',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
}

/**
 * 恢复备份
 * @param params
 * @param params.templateId - 目标模板 ID
 * @param params.file - 备份文件 (.tsbk)
 * @returns Promise<RestoreBackupResponse>
 */
export function restoreBackup(params: {
  templateId: number;
  file: File;
}): Promise<RestoreBackupResponse> {
  const formData = new FormData();
  formData.append('templateId', String(params.templateId));
  formData.append('backupFile', params.file);

  return request({
    url: '/api/v1/backup/restore',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
}

// =============== 类型定义 ===============

/**
 * 备份清单
 */
export interface BackupManifest {
  version: string;
  format: string;
  createdAt: string;
  templateId: number;
  templateName: string;
  checksum: string;
  filesChecksum: Record<string, string>;
}

/**
 * 预览备份响应
 */
export interface BackupPreviewResponse {
  manifest: BackupManifest;
  templateName: string;
  fileCount: number;
  hasVariables: boolean;
  hasTestData: boolean;
  hasConditions: boolean;
  filePaths: string[];
  checksumValid: boolean;
}

/**
 * 恢复统计
 */
export interface RestoreStats {
  filesRestored: number;
  variablesRestored: number;
  conditionsRestored: number;
  testDataRestored: boolean;
}

/**
 * 恢复备份响应
 */
export interface RestoreBackupResponse {
  success: boolean;
  error?: string;
  stats?: RestoreStats;
}
