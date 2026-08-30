-- 用户表添加个人简介字段
-- 原 020 迁移以 main.rs 运行时 ALTER 实现，此文件将其纳入迁移目录管理
ALTER TABLE users
    ADD COLUMN bio VARCHAR(200) DEFAULT '' AFTER avatar;
