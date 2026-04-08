<template>
  <div class="step-panel">
    <div class="template-detail">
      <div class="detail-header">
        <div class="detail-title-row">
          <h2 class="detail-name">{{ template?.name }}</h2>
          <a-tag v-if="template?.categoryId" color="purple" size="large">
            {{ getCategoryName(template.categoryId) }}
          </a-tag>
          <a-tag v-if="template?.isFeatured === 1" color="gold" size="large">
            <template #icon><StarOutlined /></template>
            推荐
          </a-tag>
        </div>
        <a-divider />
        <p class="detail-desc">{{ template?.description || '暂无描述' }}</p>
      </div>

      <div v-if="versionList.length > 0" class="detail-section">
        <h3 class="detail-section-title"><TagsOutlined /> 选择版本</h3>
        <a-select
          :value="selectedVersion"
          :options="versionOptions"
          size="large"
          style="width: 100%"
          placeholder="选择版本"
          @update:value="$emit('update:selectedVersion', $event)"
        >
          <template #suffixIcon><DownloadOutlined /></template>
        </a-select>
        <div class="form-hint">当前选择：{{ selectedVersion || '未选择' }}</div>
      </div>
      <a-alert
        v-else-if="!loadingVersions"
        message="该模板暂未开放使用"
        description="此模板正在准备中，请稍后再来"
        type="info"
        show-icon
        style="margin-bottom: 24px"
      >
        <template #icon><InfoCircleOutlined /></template>
      </a-alert>
      <div v-if="loadingVersions" class="loading-ct">
        <a-spin size="small"><template #description>加载版本列表...</template></a-spin>
      </div>

      <div class="detail-section">
        <h3 class="detail-section-title"><CodeOutlined /> 支持的语言</h3>
        <div class="languages-list">
          <a-tag
            v-for="lang in template?.languages || []"
            :key="lang.id"
            :color="lang.isPrimary === 1 ? 'blue' : 'default'"
            size="large"
          >
            {{ getLanguageName(lang.languageId) }}
            <span v-if="lang.isPrimary === 1">(主语言)</span>
          </a-tag>
          <span v-if="!template?.languages?.length" class="no-lang">暂无语言信息</span>
        </div>
      </div>

      <div v-if="template?.introduction" class="detail-section">
        <h3 class="detail-section-title"><FileTextOutlined /> 详细介绍</h3>
        <div class="intro-markdown" v-html="renderedIntro"></div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  CodeOutlined, FileTextOutlined,
  StarOutlined, TagsOutlined, InfoCircleOutlined, DownloadOutlined
} from '@ant-design/icons-vue'

const props = defineProps({
  template: Object,
  versionList: Array,
  versionOptions: Array,
  selectedVersion: String,
  loadingVersions: Boolean,
  categories: Array,
  languages: Array,
  renderedIntro: String,
})

defineEmits(['update:selectedVersion'])

const getCategoryName = (categoryId) => props.categories.find(c => c.id === categoryId)?.name || categoryId
const getLanguageName = (languageId) => props.languages.find(l => l.id === languageId)?.name || languageId
</script>

<style scoped>
.step-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.template-detail {
  flex: 1;
  overflow-y: auto;
  padding: 32px 48px;
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
}

.detail-header {
  margin-bottom: 24px;
}

.detail-title-row {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.detail-name {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--color-text);
}

.detail-desc {
  font-size: 15px;
  color: var(--color-text-secondary);
  line-height: 1.8;
  margin: 0;
}

.detail-section {
  margin-bottom: 28px;
}

.detail-section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 16px 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.form-hint {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-top: 6px;
}

.languages-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.no-lang {
  font-size: 13px;
  color: var(--color-text-muted);
}

.intro-markdown {
  font-size: 14px;
  line-height: 1.8;
  color: var(--color-text-secondary);
  max-height: 400px;
  overflow-y: auto;
}

.intro-markdown :deep(h1),
.intro-markdown :deep(h2),
.intro-markdown :deep(h3),
.intro-markdown :deep(h4) {
  color: var(--color-text);
  margin: 16px 0 8px;
  font-weight: 600;
}

.intro-markdown :deep(h1) { font-size: 20px; }
.intro-markdown :deep(h2) { font-size: 18px; }
.intro-markdown :deep(h3) { font-size: 16px; }

.intro-markdown :deep(p) {
  margin: 8px 0;
}

.intro-markdown :deep(code) {
  background: var(--color-bg-elevated);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 13px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.intro-markdown :deep(pre) {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  padding: 12px 16px;
  overflow-x: auto;
  margin: 12px 0;
}

.intro-markdown :deep(pre code) {
  background: none;
  padding: 0;
}

.intro-markdown :deep(ul),
.intro-markdown :deep(ol) {
  padding-left: 20px;
  margin: 8px 0;
}

.intro-markdown :deep(li) {
  margin: 4px 0;
}

.intro-markdown :deep(blockquote) {
  border-left: 3px solid var(--color-primary);
  padding-left: 12px;
  margin: 12px 0;
  color: var(--color-text-muted);
}

.intro-markdown :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
}

.intro-markdown :deep(a:hover) {
  text-decoration: underline;
}

.intro-markdown :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 12px 0;
}

.intro-markdown :deep(th),
.intro-markdown :deep(td) {
  border: 1px solid var(--color-border);
  padding: 8px 12px;
  text-align: left;
}

.intro-markdown :deep(th) {
  background: var(--color-bg-elevated);
  font-weight: 600;
}

.intro-markdown :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 16px 0;
}

.loading-ct {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
