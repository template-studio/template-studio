<template>
  <div class="help-view">
    <a-layout class="help-layout">
      <!-- 左侧目录导航 -->
      <a-layout-sider
        v-model:collapsed="menuCollapsed"
        :collapsed-width="48"
        :width="220"
        collapsible
        :trigger="null"
        class="help-sider"
        :theme="themeStore.isDark ? 'dark' : 'light'"
      >
        <div class="sider-header">
          <span v-if="!menuCollapsed" class="sider-title">帮助中心</span>
          <a-button type="text" size="small" class="collapse-btn" @click="menuCollapsed = !menuCollapsed">
            <template #icon>
              <MenuFoldOutlined v-if="!menuCollapsed" />
              <MenuUnfoldOutlined v-else />
            </template>
          </a-button>
        </div>
        <a-menu
          v-model:selectedKeys="selectedKeys"
          v-model:openKeys="openKeys"
          mode="inline"
          class="help-menu"
        >
          <a-menu-item key="quickstart">
            <template #icon><RocketOutlined /></template>
            快速开始
          </a-menu-item>

          <a-sub-menu key="guide">
            <template #icon><BookOutlined /></template>
            <template #title>功能指南</template>
            <a-menu-item key="guide-templates">模板管理</a-menu-item>
            <a-menu-item key="guide-datasource">数据源管理</a-menu-item>
            <a-menu-item key="guide-project">项目管理</a-menu-item>
            <a-menu-item key="guide-mappings">类型映射</a-menu-item>
            <a-menu-item key="guide-render">模板渲染</a-menu-item>
            <a-menu-item key="guide-settings">设置</a-menu-item>
          </a-sub-menu>

          <a-sub-menu key="faq">
            <template #icon><QuestionCircleOutlined /></template>
            <template #title>常见问题</template>
            <a-menu-item key="faq-1">如何连接远程数据库</a-menu-item>
            <a-menu-item key="faq-2">如何导入已有表结构</a-menu-item>
            <a-menu-item key="faq-3">如何自定义生成模板</a-menu-item>
            <a-menu-item key="faq-4">如何切换暗黑模式</a-menu-item>
            <a-menu-item key="faq-5">表结构对比无法同步</a-menu-item>
          </a-sub-menu>

          <a-menu-item key="shortcuts">
            <template #icon><ThunderboltOutlined /></template>
            快捷键参考
          </a-menu-item>
        </a-menu>
      </a-layout-sider>

      <!-- 右侧内容区 -->
      <a-layout-content class="help-content">
        <div class="content-wrapper">

          <!-- 快速开始 -->
          <section v-if="selectedKeys[0] === 'quickstart'" class="content-section">
            <h1><RocketOutlined /> 快速开始</h1>
            <p class="section-desc">3 步骤快速上手 Template Studio</p>
            <a-row :gutter="16" class="quickstart-cards">
              <a-col :span="8">
                <div class="quick-step">
                  <div class="step-number">1</div>
                  <div class="step-icon"><CloudServerOutlined /></div>
                  <h3>创建数据源</h3>
                  <p>配置 MySQL、PostgreSQL 或 SQLite 数据库连接，测试连接状态</p>
                </div>
              </a-col>
              <a-col :span="8">
                <div class="quick-step">
                  <div class="step-number">2</div>
                  <div class="step-icon"><TableOutlined /></div>
                  <h3>管理项目表</h3>
                  <p>定义表结构、列信息，支持拖拽排序、从数据库导入、AI 建表</p>
                </div>
              </a-col>
              <a-col :span="8">
                <div class="quick-step">
                  <div class="step-number">3</div>
                  <div class="step-icon"><CodeOutlined /></div>
                  <h3>生成代码</h3>
                  <p>使用模板渲染功能，配置变量后预览并导出生成的代码文件</p>
                </div>
              </a-col>
            </a-row>
          </section>

          <!-- 功能指南 -->
          <section v-if="selectedKeys[0] === 'guide-templates'" class="content-section">
            <h1><BookOutlined /> 模板管理</h1>
            <div class="guide-content">
              <ul>
                <li>浏览模板库，支持按分类、语言筛选和搜索</li>
                <li>点击模板卡片查看详情，选择版本后进入配置向导</li>
                <li>填写变量表单，支持普通模式和高级 JSON 模式</li>
                <li>预览渲染结果后导出到指定目录</li>
              </ul>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'guide-datasource'" class="content-section">
            <h1><CloudServerOutlined /> 数据源管理</h1>
            <div class="guide-content">
              <ul>
                <li>添加 MySQL、PostgreSQL 或 SQLite 数据源</li>
                <li>测试连接状态，查看服务器版本、延迟、连接池信息</li>
                <li>使用数据库浏览器查看表数据和列信息</li>
                <li>支持连接池缓存，避免重复建立连接</li>
              </ul>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'guide-project'" class="content-section">
            <h1><TableOutlined /> 项目管理</h1>
            <div class="guide-content">
              <ul>
                <li>创建项目并关联数据源</li>
                <li>管理表结构：添加列、拖拽排序、批量删除</li>
                <li>从数据库导入已有表结构，或使用 AI 自然语言建表</li>
                <li>表结构对比与同步：查看本地与远程表的差异，双向同步</li>
                <li>导出表的 SQL DDL 语句</li>
              </ul>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'guide-mappings'" class="content-section">
            <h1><SwapOutlined /> 类型映射</h1>
            <div class="guide-content">
              <ul>
                <li>配置数据库类型到编程语言类型的映射关系</li>
                <li>支持项目级和全局级映射配置</li>
                <li>导入/导出映射配置（JSON 格式）</li>
                <li>使用预置模板快速配置（MySQL→Java MyBatis/JPA 等）</li>
              </ul>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'guide-render'" class="content-section">
            <h1><CodeOutlined /> 模板渲染</h1>
            <div class="guide-content">
              <ul>
                <li>选择已下载的模板，查看详细介绍和版本信息</li>
                <li>配置模板变量（普通模式表单或高级模式 JSON 编辑器）</li>
                <li>注入上下文信息（项目信息、表信息）</li>
                <li>预览渲染结果（文件树 + 代码高亮）</li>
                <li>导出到指定目录</li>
              </ul>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'guide-settings'" class="content-section">
            <h1><SettingOutlined /> 设置</h1>
            <div class="guide-content">
              <ul>
                <li>显示设置：主题模式（亮/暗）、主题颜色、字体大小、动画效果</li>
                <li>快捷键配置：10 个常用快捷键，支持自定义录制</li>
                <li>AI 服务配置：配置 AI 提供商和模型，测试连接</li>
                <li>备份与路径：数据导出/导入、模板路径、自动备份</li>
              </ul>
            </div>
          </section>

          <!-- 常见问题 -->
          <section v-if="selectedKeys[0] === 'faq-1'" class="content-section">
            <h1><QuestionCircleOutlined /> 如何连接远程数据库？</h1>
            <div class="faq-content">
              <p>进入「数据源」页面，点击「新建数据源」，选择数据库类型（MySQL/PostgreSQL/SQLite），填写连接信息后点击「测试连接」确认无误即可保存。</p>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'faq-2'" class="content-section">
            <h1><QuestionCircleOutlined /> 如何导入已有的表结构？</h1>
            <div class="faq-content">
              <p>在项目的表管理页面，点击「从数据库导入」按钮，选择已配置的数据源，勾选需要导入的表即可。也支持通过 SQL 语句导入表结构。</p>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'faq-3'" class="content-section">
            <h1><QuestionCircleOutlined /> 如何自定义代码生成模板？</h1>
            <div class="faq-content">
              <p>进入「设置 &gt; 常规」，配置模板存储路径。模板遵循 MiniJinja 语法，变量定义在 <code>.meta/variables/variables.json</code> 文件中。</p>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'faq-4'" class="content-section">
            <h1><QuestionCircleOutlined /> 如何切换暗黑模式？</h1>
            <div class="faq-content">
              <p>两种方式：</p>
              <ol>
                <li>侧边栏底部的主题切换按钮快速切换</li>
                <li>进入「设置 &gt; 显示」选择主题模式（跟随系统/亮色/暗色）</li>
              </ol>
            </div>
          </section>

          <section v-if="selectedKeys[0] === 'faq-5'" class="content-section">
            <h1><QuestionCircleOutlined /> 表结构对比显示差异但无法同步？</h1>
            <div class="faq-content">
              <p>请确认以下几点：</p>
              <ol>
                <li>远程数据库连接正常（可在数据源页面测试连接）</li>
                <li>数据库用户有 ALTER TABLE 权限</li>
                <li>差异列的数据类型在目标数据库中受支持</li>
              </ol>
            </div>
          </section>

          <!-- 快捷键参考 -->
          <section v-if="selectedKeys[0] === 'shortcuts'" class="content-section">
            <h1><ThunderboltOutlined /> 快捷键参考</h1>
            <p class="section-desc">可在「设置 &gt; 快捷键」中自定义</p>
            <div class="shortcuts-grid">
              <div v-for="s in shortcuts" :key="s.id" class="shortcut-row">
                <span class="shortcut-name">{{ s.name }}</span>
                <kbd class="shortcut-key">{{ s.key }}</kbd>
              </div>
            </div>
          </section>

        </div>
      </a-layout-content>
    </a-layout>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useThemeStore } from '@/stores/theme'
import {
  QuestionCircleOutlined, RocketOutlined, BookOutlined,
  ThunderboltOutlined, CloudServerOutlined, TableOutlined,
  CodeOutlined, SettingOutlined, SwapOutlined,
  MenuFoldOutlined, MenuUnfoldOutlined
} from '@ant-design/icons-vue'

const themeStore = useThemeStore()
const menuCollapsed = ref(false)
const selectedKeys = ref(['quickstart'])
const openKeys = ref(['guide', 'faq'])

const shortcuts = [
  { id: 'newProject', name: '新建项目', key: 'Ctrl+N' },
  { id: 'openProject', name: '打开项目', key: 'Ctrl+O' },
  { id: 'save', name: '保存', key: 'Ctrl+S' },
  { id: 'search', name: '全局搜索', key: 'Ctrl+K' },
  { id: 'toggleSidebar', name: '切换侧边栏', key: 'Ctrl+B' },
  { id: 'newTable', name: '新建表', key: 'Ctrl+T' },
  { id: 'aiGenerate', name: 'AI 生成', key: 'Ctrl+G' },
  { id: 'refresh', name: '刷新', key: 'F5' },
  { id: 'settings', name: '打开设置', key: 'Ctrl+,' },
  { id: 'closeTab', name: '关闭标签', key: 'Ctrl+W' }
]
</script>

<style scoped>
.help-view {
  height: 100%;
  overflow: hidden;
}

.help-layout {
  height: 100%;
  background: var(--color-background);
}

/* 左侧菜单 */
.help-sider {
  background: var(--color-surface) !important;
  border-right: 1px solid var(--color-border);
  overflow-y: auto;
}

.help-sider :deep(.ant-layout-sider-children) {
  display: flex;
  flex-direction: column;
}

.sider-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 8px;
  flex-shrink: 0;
}

.sider-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.collapse-btn {
  color: var(--color-text-secondary);
}

.help-menu {
  border-inline-end: none !important;
  background: transparent !important;
  flex: 1;
}

.help-menu :deep(.ant-menu-item) {
  font-size: 13px;
  height: 36px;
  line-height: 36px;
  margin: 2px 0;
}

.help-menu :deep(.ant-menu-submenu-title) {
  font-size: 13px;
  height: 38px;
  line-height: 38px;
}

.help-menu :deep(.ant-menu-item-selected) {
  background: var(--color-primary-bg) !important;
  color: var(--color-primary) !important;
}

/* 右侧内容区 */
.help-content {
  overflow-y: auto;
  background: var(--color-background);
}

.content-wrapper {
  max-width: 860px;
  margin: 0 auto;
  padding: 28px 32px 40px;
}

.content-section h1 {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text);
  margin: 0 0 8px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.section-desc {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin: 0 0 24px;
}

/* 快速开始卡片 */
.quickstart-cards {
  margin-top: 8px;
}

.quick-step {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: 24px 20px;
  text-align: center;
  position: relative;
  height: 100%;
  transition: transform var(--transition-normal), box-shadow var(--transition-normal);
}

.quick-step:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.step-number {
  position: absolute;
  top: 12px;
  left: 12px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--color-primary);
  color: #fff;
  font-size: 12px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.step-icon {
  font-size: 32px;
  color: var(--color-primary);
  margin-bottom: 12px;
}

.quick-step h3 {
  margin: 0 0 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.quick-step p {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.6;
}

/* 功能指南内容 */
.guide-content ul {
  margin: 16px 0 0;
  padding-left: 20px;
}

.guide-content li {
  font-size: 14px;
  color: var(--color-text-secondary);
  line-height: 2;
}

/* FAQ 内容 */
.faq-content {
  margin-top: 16px;
}

.faq-content p,
.faq-content li {
  font-size: 14px;
  color: var(--color-text-secondary);
  line-height: 1.8;
}

.faq-content ol {
  padding-left: 20px;
}

.faq-content code {
  background: var(--color-bg-elevated);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', monospace;
}

/* 快捷键 */
.shortcuts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  margin-top: 16px;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 8px;
}

.shortcut-name {
  font-size: 13px;
  color: var(--color-text);
}

.shortcut-key {
  display: inline-block;
  padding: 2px 8px;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.6;
}
</style>
