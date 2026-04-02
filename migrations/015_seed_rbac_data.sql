-- 默认管理员账号: admin / admin123 (bcrypt hash)
INSERT INTO users (username, password_hash, email, status) VALUES
('admin', '$2b$12$LJ3m4ys3NzBJSdVg8VPVMuHCFsDGZbsSSTjGBnpfEjJGvOLMKeWm6', 'admin@templatestudio.com', 1)
ON DUPLICATE KEY UPDATE username = VALUES(username);

-- 角色定义
INSERT INTO roles (name, display_name, description, sort) VALUES
('super_admin', '超级管理员', '拥有所有权限', 0),
('admin', '管理员', '常规管理权限', 1),
('viewer', '观察者', '只读权限', 2)
ON DUPLICATE KEY UPDATE display_name = VALUES(display_name);

-- 菜单权限（与前端路由对应）
INSERT INTO permissions (name, display_name, type, sort) VALUES
('dashboard', '仪表盘', 'menu', 0),
('template', '模板管理', 'menu', 1),
('category', '分类管理', 'menu', 2),
('language', '语言管理', 'menu', 3),
('var_preset', '变量预设', 'menu', 4),
('settings', '系统设置', 'menu', 5),
('user_management', '用户管理', 'menu', 6),
('role_management', '角色管理', 'menu', 7)
ON DUPLICATE KEY UPDATE display_name = VALUES(display_name);

-- 超级管理员拥有所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p WHERE r.name = 'super_admin'
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id);

-- 管理员拥有除用户/角色管理外的权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r CROSS JOIN permissions p
WHERE r.name = 'admin' AND p.name NOT IN ('user_management', 'role_management')
ON DUPLICATE KEY UPDATE role_id = VALUES(role_id);

-- admin 用户分配超级管理员角色
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id FROM users u CROSS JOIN roles r
WHERE u.username = 'admin' AND r.name = 'super_admin'
ON DUPLICATE KEY UPDATE user_id = VALUES(user_id);
