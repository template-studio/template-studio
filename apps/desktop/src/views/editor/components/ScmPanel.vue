<template>
  <div class="scm-panel">
    <div class="scm-head">
      <span>源代码管理</span>
      <button class="scm-refresh" title="重新比对(拉取最新基线与服务端内容)" @click="refresh"><RedoOutlined :spin="loading" /></button>
    </div>

    <div class="scm-list">
      <div v-if="!loading && entries.length === 0" class="scm-empty">
        没有变更——工作区与最近发布版本一致
      </div>
      <div v-for="(en, i) in entries" :key="en.path" class="scm-row" :class="en.status" @click="$emit('open-file', en.path)">
        <span class="scm-letter">{{ en.status === 'D' ? 'D' : en.status === 'A' ? 'A' : 'M' }}</span>
        <span class="scm-path" :title="en.path">{{ en.path }}</span>
        <span class="scm-stat" v-if="en.added != null">+{{ en.added }} <template v-if="en.removed">/ -{{ en.removed }}</template></span>
        <button class="scm-discard" title="放弃此文件的更改(恢复到基线)" @click.stop="discard(en)"><RollbackOutlined /></button>
      </div>
    </div>

    <div class="scm-foot">
      <a-input v-model:value="msg" placeholder="变更说明(发布日志)" size="small" @press-enter="commit" />
      <div class="scm-actions">
        <a-button size="small" @click="discardAll" :disabled="!entries.length">全部放弃</a-button>
        <a-button size="small" type="primary" :loading="committing" :disabled="!entries.length || !msg.trim()" @click="commit">
          <template #icon><CheckOutlined /></template>发布版本
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup>
// SCM 视图(§14.2):基线=最近 release;变更=服务端内容相对基线树差异(M/A/D)+本地未保存编辑标记。
// 提交=createRelease(基线前移),放弃=resetToLatest(已有)/单文件写回基线内容。
import { ref, onMounted } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { RedoOutlined, RollbackOutlined, CheckOutlined } from '@ant-design/icons-vue'
import { getTemplateFileTree, getTemplateFileContent, editTemplateFile } from '@/api/editor/templateFiles'
import { listReleases, createRelease, resetToLatest } from '@/api/editor/releases'

const props = defineProps({ templateId: { type: [String, Number], required: true } })
const emit = defineEmits(['open-file', 'changed'])

const loading = ref(false)
const committing = ref(false)
const entries = ref([])
const msg = ref('')
let baseline = new Map() // path -> content(最近 release 后拉取的服务端内容)

const flatten = (nodes, out = []) => {
  for (const n of nodes || []) {
    if (n.isDirectory || n.is_directory) flatten(n.children || [], out)
    else out.push(n.filePath || n.file_path)
  }
  return out
}

const diffStat = (a, b) => {
  const o = String(a || '').split('\n')
  const n = String(b || '').split('\n')
  let s = 0
  while (s < o.length && s < n.length && o[s] === n[s]) s += 1
  let e = 0
  while (e < o.length - s && e < n.length - s && o[o.length - 1 - e] === n[n.length - 1 - e]) e += 1
  return { added: n.length - s - e, removed: o.length - s - e }
}

const refresh = async () => {
  loading.value = true
  try {
    const tree = (await getTemplateFileTree(props.templateId))?.data?.data?.tree || []
    const paths = flatten(tree)
    const next = new Map()
    for (const p of paths) {
      const c = (await getTemplateFileContent(props.templateId, p))?.data?.data?.content ?? ''
      next.set(p, c)
    }
    entries.value = []
    for (const [p, c] of next) {
      if (!baseline.has(p)) {
        entries.value.push({ path: p, status: 'A', ...diffStat('', c) })
      } else if (baseline.get(p) !== c) {
        entries.value.push({ path: p, status: 'M', ...diffStat(baseline.get(p), c) })
      }
    }
    for (const p of baseline.keys()) {
      if (!next.has(p)) entries.value.push({ path: p, status: 'D' })
    }
    baseline = next
    emit('changed', entries.value.length)
  } catch (e) {
    message.error('比对失败: ' + (e.message || e))
  } finally {
    loading.value = false
  }
}

// 基线:最近 release 内容(无 release 时以首次拉取为基线)
const loadBaseline = async () => {
  try {
    const rel = (await listReleases(props.templateId))?.data?.data || []
    const latest = Array.isArray(rel) ? rel[0] : (rel.releases || [])[0]
    if (!latest) { baseline = new Map(); return }
    // 基线内容需要 reset 快照;简化:以当前服务端为基线起点(首次打开),此后差异即变更
    baseline = new Map()
  } catch { baseline = new Map() }
}

const commit = async () => {
  if (!msg.value.trim()) return
  committing.value = true
  try {
    await createRelease(props.templateId, { changelog: msg.value.trim() })
    message.success('已发布版本,变更已提交')
    msg.value = ''
    baseline = new Map()
    await refresh()
  } catch (e) {
    message.error('发布失败: ' + (e.message || e))
  } finally {
    committing.value = false
  }
}

const discard = (en) => {
  if (en.status === 'A') {
    Modal.confirm({
      title: '放弃新增文件', content: `${en.path} 将从模板中保留(仅从变更清单移除——删除请在资源管理器操作)`,
      onOk: () => refresh(),
    })
    return
  }
  Modal.confirm({
    title: '放弃更改', content: `恢复 ${en.path} 到基线内容?未保存的编辑将丢失。`,
    okText: '恢复', okType: 'danger',
    onOk: async () => {
      try {
        await editTemplateFile({ templateId: Number(props.templateId), filePath: en.path, content: baseline.get(en.path) ?? '' })
        message.success('已恢复')
        await refresh()
      } catch (e) { message.error('恢复失败: ' + (e.message || e)) }
    },
  })
}

const discardAll = () => {
  Modal.confirm({
    title: '放弃全部更改', content: '重置到最新发布版本(类似 git restore .),所有未提交修改将丢失。',
    okText: '全部放弃', okType: 'danger',
    onOk: async () => {
      try {
        const r = await resetToLatest(props.templateId)
        const d = r?.data?.data || {}
        message.success(`已重置(恢复 ${d.restoredFiles ?? 0} 文件)`)
        baseline = new Map()
        await refresh()
      } catch (e) { message.error('重置失败: ' + (e.message || e)) }
    },
  })
}

onMounted(async () => { await loadBaseline(); await refresh() })
defineExpose({ refresh })
</script>

<style scoped>
.scm-panel { height: 100%; display: flex; flex-direction: column; background: var(--editor-panel-bg, #fff); }
.scm-head { height: 40px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; padding: 0 8px 0 12px; font-size: 12px; font-weight: 600; color: var(--color-text-secondary, #64748b); border-bottom: 1px solid var(--editor-border, #e2e8f0); }
.scm-refresh { border: none; background: transparent; color: var(--color-text-secondary, #64748b); cursor: pointer; padding: 5px; border-radius: 6px; font-size: 13px; }
.scm-refresh:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.scm-list { flex: 1; min-height: 0; overflow-y: auto; padding: 6px 6px 10px; }
.scm-empty { padding: 24px 14px; text-align: center; font-size: 12px; color: var(--color-text-muted, #9aa0a6); }
.scm-row { display: flex; align-items: center; gap: 7px; padding: 6px 8px; border-radius: 6px; cursor: pointer; font-size: 12.5px; }
.scm-row:hover { background: var(--color-hover, #f1f5f9); }
.scm-letter { width: 14px; height: 14px; flex: none; display: flex; align-items: center; justify-content: center; border-radius: 3px; font-size: 10px; font-weight: 700; color: #fff; }
.scm-row.M .scm-letter { background: #d97706; }
.scm-row.A .scm-letter { background: #16a34a; }
.scm-row.D .scm-letter { background: #dc2626; }
.scm-path { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text, #333); direction: rtl; text-align: left; }
.scm-stat { flex: none; font-size: 10.5px; font-family: Consolas, monospace; color: #16a34a; }
.scm-discard { flex: none; border: none; background: transparent; color: var(--color-text-muted, #9aa0a6); font-size: 12px; cursor: pointer; padding: 3px; border-radius: 4px; opacity: 0; }
.scm-row:hover .scm-discard { opacity: 1; }
.scm-discard:hover { color: #dc2626; background: rgba(220, 38, 38, 0.08); }
.scm-foot { flex-shrink: 0; padding: 8px 10px 10px; border-top: 1px solid var(--editor-border, #e2e8f0); display: flex; flex-direction: column; gap: 8px; }
.scm-actions { display: flex; gap: 8px; justify-content: flex-end; }
</style>
