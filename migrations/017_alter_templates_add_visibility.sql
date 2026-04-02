-- 模板投稿系统：templates 表新增字段
ALTER TABLE templates
  ADD COLUMN owner_id BIGINT DEFAULT NULL COMMENT '模板所有者用户ID，NULL表示系统模板',
  ADD COLUMN visibility VARCHAR(20) DEFAULT 'public' COMMENT 'private=私有 draft=草稿 pending=待审核 public=公开',
  ADD COLUMN status VARCHAR(20) DEFAULT 'active' COMMENT 'active=正常 rejected=被拒 disabled=下架',
  ADD COLUMN reviewed_at DATETIME DEFAULT NULL COMMENT '审核时间',
  ADD COLUMN reviewed_by BIGINT DEFAULT NULL COMMENT '审核人ID',
  ADD COLUMN download_count INT DEFAULT 0 COMMENT '下载/使用次数';

-- 索引
ALTER TABLE templates ADD INDEX idx_owner_id (owner_id);
ALTER TABLE templates ADD INDEX idx_visibility (visibility);
ALTER TABLE templates ADD INDEX idx_status (status);

-- 现有模板全部设为系统公开模板（owner_id=NULL, visibility=public）
UPDATE templates SET visibility = 'public', status = 'active' WHERE owner_id IS NULL;
