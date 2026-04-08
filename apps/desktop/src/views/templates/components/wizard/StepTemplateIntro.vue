<template>
  <div class="step-panel">
    <div class="template-intro">
      <div class="intro-header">
        <h2>{{ wizardData.template?.name }}</h2>
        <a-tag color="purple" size="large">{{ getCategoryName(wizardData.template?.categoryId) }}</a-tag>
        <a-tag v-if="wizardData.template?.isFeatured === 1" color="gold" size="large"><template #icon><StarOutlined /></template>推荐</a-tag>
      </div>
      <a-divider />
      <p class="intro-desc">{{ wizardData.template?.description }}</p>
      <div v-if="versionList.length > 0" class="intro-section">
        <h3><TagsOutlined /> 选择版本</h3>
        <a-select :value="wizardData.version" @update:value="$emit('update:version', $event)" :options="versionOptions" size="large" style="width:100%;" @change="$emit('version-change', $event)" placeholder="选择版本"><template #suffixIcon><DownloadOutlined /></template></a-select>
        <div class="form-hint">当前选择：{{ wizardData.version || '未选择' }}</div>
      </div>
      <div class="intro-section">
        <h3><CodeOutlined /> 支持的语言</h3>
        <div class="languages-list">
          <a-tag v-for="lang in wizardData.template?.languages" :key="lang.id" :color="lang.isPrimary === 1 ? 'blue' : 'default'" size="large">{{ getLanguageName(lang.languageId) }}<span v-if="lang.isPrimary === 1">(主语言)</span></a-tag>
        </div>
      </div>
      <div v-if="wizardData.template?.introduction" class="intro-section">
        <h3><FileTextOutlined /> 详细介绍</h3>
        <div class="intro-markdown" v-html="renderedIntro"></div>
      </div>
      <a-alert v-if="versionList.length === 0 && !isDownloading" message="该模板暂未开放使用" description="此模板正在准备中，请稍后再来" type="info" show-icon style="margin-bottom:24px;"><template #icon><InfoCircleOutlined /></template></a-alert>
      <div v-if="isDownloading" class="intro-section">
        <a-alert message="请等待模板下载完成" type="info" show-icon>
          <template #icon><LoadingOutlined style="animation:spin 1s linear infinite;" /></template>
          <template #description><a-progress :percent="downloadProgress" :show-info="false" /></template>
        </a-alert>
      </div>
    </div>
  </div>
</template>

<script setup>
import { StarOutlined, TagsOutlined, DownloadOutlined, CodeOutlined, FileTextOutlined, InfoCircleOutlined, LoadingOutlined } from '@ant-design/icons-vue'

defineProps({
  wizardData: Object,
  versionList: Array,
  versionOptions: Array,
  renderedIntro: String,
  isDownloading: Boolean,
  downloadProgress: Number,
  getCategoryName: Function,
  getLanguageName: Function
})

defineEmits(['update:version', 'version-change'])
</script>

<style scoped>
.step-panel { padding: 8px; display: flex; flex-direction: column; min-height: 0; }
.template-intro { padding: 8px; }
.intro-header { display: flex; align-items: center; gap: 16px; margin-bottom: 24px; flex-wrap: wrap; }
.intro-header h2 { margin: 0; font-size: 28px; font-weight: 600; color: var(--color-text); }
.intro-desc { font-size: 16px; color: var(--color-text-secondary); line-height: 1.8; margin-bottom: 32px; }
.intro-section { margin-bottom: 32px; }
.intro-section h3 { font-size: 18px; font-weight: 600; color: var(--color-text); margin-bottom: 16px; display: flex; align-items: center; gap: 8px; }
.languages-list { display: flex; flex-wrap: wrap; gap: 12px; }
.form-hint { font-size: 12px; color: var(--color-text-secondary); margin-top: 4px; }
.intro-markdown { line-height: 1.8; color: var(--color-text-secondary); font-size: 14px; max-height: 400px; overflow-y: auto; }
.intro-markdown :deep(h1), .intro-markdown :deep(h2), .intro-markdown :deep(h3), .intro-markdown :deep(h4) { color: var(--color-text); margin: 16px 0 8px; font-weight: 600; }
.intro-markdown :deep(h1) { font-size: 20px; } .intro-markdown :deep(h2) { font-size: 18px; } .intro-markdown :deep(h3) { font-size: 16px; }
.intro-markdown :deep(p) { margin: 8px 0; }
.intro-markdown :deep(code) { background: var(--color-bg-elevated); padding: 2px 6px; border-radius: 4px; font-size: 13px; font-family: 'Consolas', 'Monaco', monospace; }
.intro-markdown :deep(pre) { background: var(--color-bg-elevated); border: 1px solid var(--color-border); border-radius: 6px; padding: 12px 16px; overflow-x: auto; margin: 12px 0; }
.intro-markdown :deep(pre code) { background: none; padding: 0; }
.intro-markdown :deep(ul), .intro-markdown :deep(ol) { padding-left: 20px; margin: 8px 0; }
.intro-markdown :deep(li) { margin: 4px 0; }
.intro-markdown :deep(blockquote) { border-left: 3px solid var(--color-primary); padding-left: 12px; margin: 12px 0; color: var(--color-text-muted); }
.intro-markdown :deep(a) { color: var(--color-primary); text-decoration: none; }
.intro-markdown :deep(table) { border-collapse: collapse; width: 100%; margin: 12px 0; }
.intro-markdown :deep(th), .intro-markdown :deep(td) { border: 1px solid var(--color-border); padding: 8px 12px; text-align: left; }
.intro-markdown :deep(th) { background: var(--color-bg-elevated); font-weight: 600; }
.intro-markdown :deep(hr) { border: none; border-top: 1px solid var(--color-border); margin: 16px 0; }
</style>
