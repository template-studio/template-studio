<template>
  <div class="convert-guide">
    <!-- 顶部工具栏(与模板库同款) -->
    <div class="toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">项目转换</h2>
        <span class="result-count">共 {{ drafts.length }} 个转换草稿</span>
      </div>
      <div class="toolbar-right">
        <a-button type="primary" @click="sourceOpen = true">
          <template #icon><PlusOutlined /></template>
          新建转换
        </a-button>
      </div>
    </div>

    <!-- 卡片网格:新建 + 草稿 -->
    <div class="guide-content">
      <a-spin :spinning="loading">
        <div class="cards-grid">
          <div class="g-card new-card" @click="sourceOpen = true">
            <div class="new-inner">
              <div class="new-ico"><PlusOutlined /></div>
              <div class="new-title">新建转换</div>
              <div class="new-sub">git 仓库克隆 → AI 分析 → 模板化</div>
            </div>
          </div>

          <div v-for="d in drafts" :key="d.id" class="g-card draft-card" @click="continueDraft(d.id)">
            <div class="card-visual">
              <div class="visual-bg"><BranchesOutlined class="draft-bg-ico" /></div>
              <div v-if="srcOf(d).packId" class="draft-badge">{{ srcOf(d).packId }}</div>
            </div>
            <div class="card-content">
              <h3 class="draft-name" :title="srcOf(d).source">{{ nameOf(d) }}</h3>
              <p class="draft-src">{{ srcOf(d).source || d.id }}</p>
              <div class="draft-tags">
                <span v-if="srcOf(d).branch" class="template-tag">{{ srcOf(d).branch }}</span>
                <span v-if="srcOf(d).commit" class="template-tag mono">{{ srcOf(d).commit.slice(0, 7) }}</span>
                <span v-if="d.meta?.degraded" class="template-tag warn">降级</span>
              </div>
              <div class="card-footer">
                <span class="creation-time">{{ fmtTime(d.mtimeMs) }}</span>
                <div class="card-footer-right">
                  <a-tooltip title="继续转换">
                    <a-button type="text" size="small" class="cont-entry" @click.stop="continueDraft(d.id)">
                      <template #icon><RightOutlined /></template>
                    </a-button>
                  </a-tooltip>
                  <a-tooltip title="删除草稿">
                    <a-button type="text" size="small" class="del-entry" @click.stop="removeDraft(d.id)">
                      <template #icon><DeleteOutlined /></template>
                    </a-button>
                  </a-tooltip>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="!loading && drafts.length === 0" class="guide-hint">
          还没有进行中的转换——点击「新建转换」,从 git 仓库克隆项目开始模板化
        </div>
      </a-spin>
    </div>

    <!-- 来源选择弹窗(与模板页「新建模板 → 从项目转换」共用) -->
    <ConvertSourceModal v-model:open="sourceOpen" @start="onStart" />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { PlusOutlined, DeleteOutlined, RightOutlined, BranchesOutlined } from '@ant-design/icons-vue'
import ConvertSourceModal from './components/ConvertSourceModal.vue'

const router = useRouter()
const sourceOpen = ref(false)
const loading = ref(false)
const drafts = ref([])

const srcOf = (d) => d.meta?.source || {}
const nameOf = (d) => {
  const s = srcOf(d).source || ''
  return s.split(/[\\/]/).pop()?.replace(/\.git$/, '') || '未命名转换'
}

const loadDrafts = async () => {
  loading.value = true
  try {
    const raw = await invoke('convert_draft_list')
    drafts.value = JSON.parse(raw).items || []
  } catch {
    drafts.value = []
  } finally {
    loading.value = false
  }
}

const onStart = ({ source, branch }) => {
  sourceOpen.value = false
  router.push({ path: '/convert/workbench', query: { src: source, ...(branch ? { branch } : {}) } })
}

const continueDraft = (id) => {
  router.push({ path: '/convert/workbench', query: { draft: id } })
}

const removeDraft = async (id) => {
  try {
    await invoke('convert_draft_delete', { id })
    await loadDrafts()
  } catch (e) {
    message.error('删除失败: ' + (e.message || e))
  }
}

const fmtTime = (ms) => {
  const d = new Date(Number(ms))
  if (Number.isNaN(d.getTime())) return ''
  const now = new Date()
  const diff = now - d
  if (diff < 60_000) return '刚刚'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`
  if (d.toDateString() === now.toDateString()) return '今天'
  return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}

onMounted(loadDrafts)
</script>

<style scoped>
.convert-guide { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
.toolbar { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--spacing-md); padding: var(--spacing-sm) var(--spacing-lg); flex-shrink: 0; }
.toolbar-left { display: flex; align-items: baseline; gap: var(--spacing-md); }
.page-title { margin: 0; font-size: 24px; font-weight: 600; color: var(--color-text); }
.result-count { color: var(--color-text-secondary); font-size: 14px; }
.toolbar-right { display: flex; align-items: center; gap: var(--spacing-md); }

.guide-content { flex: 1; overflow-y: auto; min-height: 0; padding: 4px var(--spacing-lg) 24px; }
.guide-content :deep(.ant-spin-nested-loading),
.guide-content :deep(.ant-spin-container) { min-height: 320px; }

.cards-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: var(--spacing-md); }
@media (max-width: 1400px) { .cards-grid { grid-template-columns: repeat(4, 1fr); } }
@media (max-width: 1100px) { .cards-grid { grid-template-columns: repeat(3, 1fr); } }

/* 新建卡:虚线占位 */
.g-card { background: var(--color-background); border: 1px solid var(--color-border); border-radius: var(--border-radius-lg); overflow: hidden; cursor: pointer; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); }
.new-card { border: 1.5px dashed var(--color-border); display: flex; align-items: center; justify-content: center; min-height: 246px; background: transparent; }
.new-card:hover { border-color: var(--color-brand, #16a34a); background: rgba(22, 163, 74, 0.03); transform: translateY(-4px); box-shadow: 0 12px 32px rgba(15, 23, 42, 0.08); }
.new-inner { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 24px; text-align: center; }
.new-ico { width: 44px; height: 44px; border-radius: 12px; background: var(--color-surface, #f1f1ef); display: flex; align-items: center; justify-content: center; font-size: 18px; color: var(--color-brand, #16a34a); }
.new-title { font-size: 15px; font-weight: 600; color: var(--color-text); }
.new-sub { font-size: 12px; color: var(--color-text-secondary); }

/* 草稿卡:模板卡同构 */
.draft-card:hover { transform: translateY(-4px); box-shadow: 0 12px 32px rgba(15, 23, 42, 0.12); border-color: var(--color-border-strong); }
.card-visual { height: 110px; position: relative; overflow: hidden; }
.visual-bg { width: 100%; height: 100%; background: var(--cover-gradient); display: flex; align-items: center; justify-content: center; }
.draft-bg-ico { font-size: 34px; color: rgba(148, 163, 184, 0.45); }
.draft-badge { position: absolute; top: 10px; right: 10px; background: rgba(22, 163, 74, 0.9); padding: 3px 10px; border-radius: 6px; font-size: 11px; font-weight: 600; color: #fff; letter-spacing: 0.3px; text-transform: uppercase; }
.card-content { padding: 14px 16px 16px; display: flex; flex-direction: column; }
.draft-name { margin: 0 0 4px 0; font-size: 15px; font-weight: 600; color: var(--color-text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.draft-card:hover .draft-name { color: var(--color-primary); }
.draft-src { margin: 0 0 10px 0; font-size: 12px; color: var(--color-text-secondary); font-family: Consolas, 'JetBrains Mono', monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: rtl; text-align: left; }
.draft-tags { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 12px; }
.template-tag { background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text-secondary); font-size: 11px; padding: 2px 8px; border-radius: 4px; }
.template-tag.mono { font-family: Consolas, monospace; }
.template-tag.warn { color: #b45309; border-color: rgba(217, 119, 6, 0.35); background: rgba(217, 119, 6, 0.06); }
.card-footer { display: flex; justify-content: space-between; align-items: center; padding-top: 10px; border-top: 1px solid var(--color-border-light); margin-top: auto; }
.creation-time { font-size: 12px; color: var(--color-text-muted); }
.card-footer-right { display: flex; align-items: center; gap: 2px; }
.cont-entry { color: var(--color-text-secondary); }
.cont-entry:hover { color: var(--color-brand, #16a34a) !important; }
.del-entry { color: var(--color-text-secondary); }
.del-entry:hover { color: #dc2626 !important; }

.guide-hint { text-align: center; color: var(--color-text-muted); font-size: 13px; padding: 48px 0 24px; }
</style>
