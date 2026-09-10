<template>
  <div class="scm-panel">
    <!-- VSCode 式:标题行(粗体小字) + 右侧悬停动作区 -->
    <div class="scm-head">
      <span class="scm-title">更改<ScmCount v-if="!loading && entries.length">({{ entries.length }})</ScmCount></span>
      <div class="scm-head-actions">
        <button class="scm-act" :title="treeMode ? '切换为列表视图' : '切换为树视图'" @click="toggleMode">
          <ApartmentOutlined v-if="!treeMode" />
          <UnorderedListOutlined v-else />
        </button>
        <button class="scm-act" title="放弃所有更改" :disabled="!entries.length" @click="discardAll"><UndoOutlined /></button>
        <button class="scm-act" title="刷新" @click="refresh"><RedoOutlined :spin="loading" /></button>
      </div>
    </div>

    <div class="scm-list">
      <div v-if="!loading && entries.length === 0" class="scm-empty">没有更改</div>

      <!-- 树视图:VSCode 式目录节段(箭头+目录名+计数,文件行缩进) -->
      <template v-if="treeMode">
        <ScmNode v-for="n in tree" :key="n.key" :node="n" :depth="0" @open-file="(p) => $emit('open-file', p)" @discard="discard" />
      </template>

      <!-- 列表视图 -->
      <div
        v-for="en in entries" v-else :key="en.path"
        class="scm-row" :class="en.status" @click="$emit('open-file', en.path)"
      >
        <span class="scm-name" :title="en.path">{{ en.path }}</span>
        <span class="scm-stat" v-if="en.added != null"><i class="a">+{{ en.added }}</i><i v-if="en.removed" class="d">-{{ en.removed }}</i></span>
        <span class="scm-letter" :title="statusName(en)">{{ en.status }}</span>
        <button class="scm-row-act" title="放弃更改" @click.stop="discard(en)"><UndoOutlined /></button>
      </div>
    </div>

    <!-- 提交框:VSCode 式输入 + 块级提交按钮 -->
    <div class="scm-foot">
      <a-textarea v-model:value="msg" placeholder="消息(按 Ctrl+Enter 提交)" :auto-size="{ minRows: 1, maxRows: 3 }" @keydown.ctrl-enter.prevent="commit" />
      <a-button block size="small" type="primary" :loading="committing" :disabled="!entries.length || !msg.trim()" style="margin-top: 6px" @click="commit">
        <template #icon><CheckOutlined /></template>提交
      </a-button>
    </div>
  </div>
</template>

<script setup>
// SCM 视图(§14.2):基线=最近 release;变更=服务端内容相对基线 M/A/D+diffstat。
// 提交=createRelease(基线前移),放弃=resetToLatest/单文件写回基线内容。
import { ref, computed, onMounted } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { RedoOutlined, UndoOutlined, CheckOutlined, UnorderedListOutlined, ApartmentOutlined } from '@ant-design/icons-vue'
import ScmNode from './ScmNode.vue'
import { getTemplateFileTree, getTemplateFileContent, editTemplateFile } from '@/api/editor/templateFiles'
import { listReleases, createRelease, resetToLatest } from '@/api/editor/releases'

const ScmCount = { props: [], template: '<span class="scm-count"><slot /></span>' }

const props = defineProps({ templateId: { type: [String, Number], required: true } })
const emit = defineEmits(['open-file', 'changed'])

const loading = ref(false)
const committing = ref(false)
const entries = ref([])
const msg = ref('')
const treeMode = ref(localStorage.getItem('scm-tree') !== '0')
const toggleMode = () => { treeMode.value = !treeMode.value; localStorage.setItem('scm-tree', treeMode.value ? '1' : '0') }

const statusName = (en) => ({ M: '已修改', A: '新增', D: '已删除' }[en.status] || en.status)

// 变更路径 → 嵌套树(仅含变更所在分支;目录节点聚合其后代变更数)
const tree = computed(() => {
  const root = { key: '', name: '', dir: true, open: true, count: 0, children: new Map(), entries: [] }
  const ensureDir = (node, seg) => {
    if (!node.children.has(seg)) node.children.set(seg, { key: node.key ? node.key + '/' + seg : seg, name: seg, dir: true, open: true, count: 0, children: new Map(), entries: [] })
    return node.children.get(seg)
  }
  for (const en of entries.value) {
    const parts = en.path.split('/')
    const base = parts.pop()
    let node = root
    for (const seg of parts) node = ensureDir(node, seg)
    const item = { ...en, base, relPath: en.path }
    node.entries.push(item)
    // 祖先链计数
    let acc = root
    acc.count += 1
    for (const seg of en.path.split('/').slice(0, -1)) {
      acc = acc.children.get(seg)
      acc.count += 1
    }
  }
  const toArr = (node) => {
    const dirs = [...node.children.values()].map(toArr)
    dirs.sort((a, b) => a.name.localeCompare(b.name))
    const files = [...node.entries].sort((a, b) => a.base.localeCompare(b.base))
    return { key: node.key || '(root)', name: node.name, count: node.count, open: node.open, dirs, files }
  }
  return [...root.children.values()].map(toArr).concat(
    [...root.entries].sort((a, b) => a.base.localeCompare(b.base)).length ? { key: '(root-files)', name: '', count: root.entries.length, open: true, dirs: [], files: [...root.entries].sort((a, b) => a.base.localeCompare(b.base)) } : []
  )
})

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

let baseline = new Map() // path -> content(基线;首次拉取快照)

const refresh = async () => {
  loading.value = true
  try {
    const tree = (await getTemplateFileTree(props.templateId))?.data?.data?.tree || []
    const paths = flatten(tree)
    const next = new Map()
    for (const p of paths) {
      try {
        const c = (await getTemplateFileContent(props.templateId, p))?.data?.data?.content ?? ''
        next.set(p, c)
      } catch { /* 单文件读取失败跳过,不炸整树 */ }
    }
    entries.value = []
    for (const [p, c] of next) {
      if (!baseline.has(p)) entries.value.push({ path: p, status: 'A', ...diffStat('', c) })
      else if (baseline.get(p) !== c) entries.value.push({ path: p, status: 'M', ...diffStat(baseline.get(p), c) })
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

const commit = async () => {
  if (!msg.value.trim()) return
  committing.value = true
  try {
    await createRelease(props.templateId, { changelog: msg.value.trim() })
    message.success('已提交并发布版本')
    msg.value = ''
    baseline = new Map()
    await refresh()
  } catch (e) {
    message.error('提交失败: ' + (e.message || e))
  } finally {
    committing.value = false
  }
}

const discard = (en) => {
  if (en.status === 'A') {
    Modal.confirm({
      title: '放弃更改', content: `${en.path} 为新增文件,从变更清单移除(删除请在资源管理器操作)`,
      onOk: () => refresh(),
    })
    return
  }
  Modal.confirm({
    title: '放弃更改', content: `恢复 ${en.path} 到基线内容?`,
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
    title: '放弃所有更改', content: '重置到最新发布版本(类似 git restore .),未提交修改将丢失。',
    okText: '放弃全部', okType: 'danger',
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

onMounted(refresh)
defineExpose({ refresh })
</script>

<style scoped>
.scm-panel { height: 100%; display: flex; flex-direction: column; background: var(--editor-panel-bg, #fff); }

/* VSCode:标题行 11px 粗体,动作图标悬停显现 */
.scm-head { height: 34px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; padding: 0 6px 0 14px; border-bottom: 1px solid var(--color-border-light, #f0f0ee); }
.scm-title { font-size: 11px; font-weight: 700; letter-spacing: 0.5px; color: var(--color-text, #1b1c1f); }
.scm-count { font-weight: 400; color: var(--color-text-muted, #9aa0a6); margin-left: 3px; }
.scm-head-actions { display: flex; align-items: center; gap: 1px; opacity: 0; transition: opacity 0.12s; }
.scm-head:hover .scm-head-actions { opacity: 1; }
.scm-act { width: 24px; height: 24px; border: none; background: transparent; border-radius: 5px; cursor: pointer; color: var(--color-text-secondary, #64748b); font-size: 13px; display: inline-flex; align-items: center; justify-content: center; }
.scm-act:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.scm-act:disabled { opacity: 0.35; cursor: not-allowed; }

.scm-list { flex: 1; min-height: 0; overflow-y: auto; padding: 4px 6px 10px; }
.scm-empty { padding: 20px 12px; text-align: center; font-size: 12px; color: var(--color-text-muted, #9aa0a6); }

/* 目录节段(VSCode 折叠组) */
.scm-dir { display: flex; align-items: center; gap: 5px; padding: 4px 6px; font-size: 11.5px; font-weight: 600; color: var(--color-text, #1b1c1f); cursor: pointer; border-radius: 5px; }
.scm-dir:hover { background: var(--color-hover, #f1f5f9); }
.scm-chev { display: inline-block; transition: transform 0.12s; color: var(--color-text-muted, #9aa0a6); font-size: 10px; }
.scm-chev.open { transform: rotate(90deg); }
.scm-dir-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.scm-dir-n { font-size: 10px; font-weight: 400; color: var(--color-text-muted, #9aa0a6); }

/* 文件行(VSCode:文件名+路径淡色,状态字母靠右色块) */
.scm-row { display: flex; align-items: center; gap: 6px; height: 26px; padding: 0 6px 0 20px; border-radius: 5px; cursor: pointer; font-size: 12.5px; }
.scm-row:hover { background: var(--color-hover, #f1f5f9); }
.scm-name { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text, #333); }
.scm-desc { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; color: var(--color-text-muted, #b0b6bc); direction: rtl; text-align: left; }
.scm-stat { flex: none; font-size: 10.5px; font-family: Consolas, monospace; display: flex; gap: 4px; }
.scm-stat .a { color: #16a34a; font-style: normal; }
.scm-stat .d { color: #dc2626; font-style: normal; }
.scm-letter { flex: none; width: 15px; height: 15px; display: inline-flex; align-items: center; justify-content: center; border-radius: 3px; font-size: 10px; font-weight: 700; color: #fff; }
.scm-row.M .scm-letter { background: #d97706; }
.scm-row.A .scm-letter { background: #16a34a; }
.scm-row.D .scm-letter { background: #dc2626; }
.scm-row-act { flex: none; width: 22px; height: 22px; border: none; background: transparent; color: var(--color-text-secondary, #64748b); font-size: 12px; cursor: pointer; border-radius: 4px; display: inline-flex; align-items: center; justify-content: center; opacity: 0; }
.scm-row:hover .scm-row-act { opacity: 1; }
.scm-row-act:hover { background: rgba(220, 38, 38, 0.1); color: #dc2626; }

/* 提交框(VSCode:输入+块级提交按钮贴底) */
.scm-foot { flex-shrink: 0; padding: 10px; border-top: 1px solid var(--color-border-light, #f0f0ee); background: var(--editor-panel-bg, #fff); }
</style>
