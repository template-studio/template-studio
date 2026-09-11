<template>
  <div class="scm-panel">
    <!-- VSCode 式:标题行(粗体小字) + 右侧悬停动作区 -->
    <div class="scm-head">
      <span class="scm-title">更改<span v-if="!loading && entries.length" class="scm-count">({{ entries.length }})</span></span>
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

      <!-- 树视图:a-tree(与资源管理器文件树同构) -->
      <a-tree
        v-if="treeMode && entries.length"
        :tree-data="scmTreeData"
        :expanded-keys="expandedKeys"
        :field-names="{ key: 'key', title: 'title', children: 'children' }"
        :selectable="false"
        @expand="onExpand"
      >
        <template #switcherIcon="{ expanded, dataRef }">
          <span v-if="dataRef?.isDir" class="scm-chev" :class="{ open: expanded }">›</span>
          <span v-else></span>
        </template>
        <template #title="opt">
          <div v-if="opt.isDir" class="scm-dir-row">
            <span class="scm-dir-name">{{ opt.title }}</span>
            <span class="scm-dir-n">{{ opt.count }}</span>
          </div>
          <div v-else class="scm-row" :class="opt.status" @click="$emit('open-file', opt.path)">
            <span class="scm-name">{{ opt.title }}</span>
            <span class="scm-stat" v-if="opt.added != null"><i class="a">+{{ opt.added }}</i><i v-if="opt.removed" class="d">-{{ opt.removed }}</i></span>
            <span class="scm-letter" :title="statusName(opt)">{{ opt.status }}</span>
            <button class="scm-row-act" title="放弃更改" @click.stop="discard(opt)"><UndoOutlined /></button>
          </div>
        </template>
      </a-tree>

      <!-- 列表视图:平铺 -->
      <template v-if="!treeMode">
      <div
        v-for="en in entries" :key="en.path"
        class="scm-row flat" :class="en.status" @click="$emit('open-file', en.path)"
      >
        <span class="scm-name" :title="en.path">{{ en.path }}</span>
        <span class="scm-stat" v-if="en.added != null"><i class="a">+{{ en.added }}</i><i v-if="en.removed" class="d">-{{ en.removed }}</i></span>
        <span class="scm-letter" :title="statusName(en)">{{ en.status }}</span>
        <button class="scm-row-act" title="放弃更改" @click.stop="discard(en)"><UndoOutlined /></button>
      </div>
      </template>
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
import { getTemplateFileContent, editTemplateFile } from '@/api/editor/templateFiles'
import { createRelease, resetToLatest, gitStatus } from '@/api/editor/releases'

const props = defineProps({ templateId: { type: [String, Number], required: true } })
const emit = defineEmits(['open-file', 'changed'])

const loading = ref(false)
const committing = ref(false)
const entries = ref([])
const msg = ref('')
const treeMode = ref(localStorage.getItem('scm-tree') !== '0')
const toggleMode = () => { treeMode.value = !treeMode.value; localStorage.setItem('scm-tree', treeMode.value ? '1' : '0') }
const expandedKeys = ref([])
const onExpand = (keys) => { expandedKeys.value = keys }

const statusName = (en) => ({ M: '已修改', A: '新增', D: '已删除' }[en.status] || en.status)

// 变更路径 → a-tree 数据(仅含变更分支;目录聚合后代变更数)
const scmTreeData = computed(() => {
  const root = { key: '', dirs: new Map(), files: [] }
  const ensureDir = (node, segs) => {
    if (!segs.length) return node
    const [seg, ...rest] = segs
    if (!node.dirs.has(seg)) node.dirs.set(seg, { key: (node.key ? node.key + '/' : '') + seg, name: seg, dirs: new Map(), files: [] })
    return ensureDir(node.dirs.get(seg), rest)
  }
  for (const en of entries.value) {
    const parts = en.path.split('/')
    const base = parts.pop()
    ensureDir(root, parts).files.push({ ...en, base })
  }
  const countOf = (n) => n.files.length + [...n.dirs.values()].reduce((s, d) => s + countOf(d), 0)
  const fileNode = (f) => ({ key: f.path, title: f.base, isDir: false, path: f.path, status: f.status, added: f.added, removed: f.removed })
  const toNode = (n) => ({
    key: n.key,
    title: n.name,
    isDir: true,
    count: countOf(n),
    children: [
      ...[...n.dirs.values()].map(toNode).sort((a, b) => a.title.localeCompare(b.title)),
      ...n.files.map(fileNode).sort((a, b) => a.title.localeCompare(b.title)),
    ],
  })
  // 顶层 = 根级目录(树节点) + 根级文件(平铺),无聚合根目录节点
  return [
    ...[...root.dirs.values()].map(toNode).sort((a, b) => a.title.localeCompare(b.title)),
    ...root.files.map(fileNode).sort((a, b) => a.title.localeCompare(b.title)),
  ]
})
// 数据就绪后默认展开全部目录(变更分支通常就两三层,全展最直观)
const syncExpand = () => {
  const keys = []
  const walk = (nodes) => nodes.forEach((n) => { if (n.isDir) { keys.push(n.key); walk(n.children || []) } })
  walk(scmTreeData.value)
  expandedKeys.value = keys
}

let baseline = new Map() // path -> content(放弃单文件恢复用;懒加载)

const refresh = async () => {
  loading.value = true
  try {
    const res = await gitStatus(props.templateId)
    const list = res?.data?.data?.entries || []
    entries.value = list.map((e) => ({ path: e.path, status: e.status }))
    syncExpand()
    emit('changed', entries.value.length)
  } catch (e) {
    const status = e?.response?.status
    const url = e?.config?.url || ''
    console.error('[SCM] git-status 失败', { status, url, baseURL: e?.config?.baseURL, data: e?.response?.data })
    const hint = status === 404 ? '(404:该服务端无此接口——确认 baseURL 指向的服务端已更新重启)' : status === 401 ? '(401:API Token 未配置或失效,见 设置→Web服务器)' : ''
    message.error('读取 git 状态失败 ' + (hint || (e.message ? ': ' + e.message : '')) + (url ? ' [' + url + ']' : ''))
  } finally {
    loading.value = false
  }
}

// 放弃单文件时懒加载基线内容(最近 release 后的工作区母本)
const ensureBaseline = async (path) => {
  if (baseline.has(path)) return baseline.get(path)
  try {
    const c = (await getTemplateFileContent(props.templateId, path))?.data?.data?.content ?? ''
    baseline.set(path, c)
  } catch { baseline.set(path, '') }
  return baseline.get(path)
}

const commit = async () => {
  if (!msg.value.trim()) return
  committing.value = true
  try {
    await createRelease(props.templateId, { changelog: msg.value.trim() })
    message.success('已提交并发布版本')
    msg.value = ''
    baseline = new Map()
    entries.value = []
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
        const content = await ensureBaseline(en.path)
        await editTemplateFile({ templateId: Number(props.templateId), filePath: en.path, content })
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
        message.success(`已重置到 ${d.version || '最新版本'}${d.deletedFiles ? `，清理 ${d.deletedFiles} 个未跟踪文件` : ''}`)
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

/* 目录行(a-tree title 内) */
.scm-dir-row { display: flex; align-items: center; gap: 6px; min-width: 0; font-size: 11.5px; font-weight: 600; color: var(--color-text, #1b1c1f); }
.scm-dir-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.scm-dir-n { font-size: 10px; font-weight: 400; color: var(--color-text-muted, #9aa0a6); }
.scm-chev { display: inline-block; transition: transform 0.12s; color: var(--color-text-muted, #9aa0a6); font-size: 10px; }
.scm-chev.open { transform: rotate(90deg); }

/* 文件行 */
.scm-row { display: flex; align-items: center; gap: 6px; height: 26px; padding: 0 4px; border-radius: 5px; cursor: pointer; font-size: 12.5px; }
.scm-row:hover { background: var(--color-hover, #f1f5f9); }
.scm-row.flat { padding: 0 8px; }
.scm-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text, #333); }
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

/* a-tree 视觉收敛(与资源管理器一致) */
.scm-list :deep(.ant-tree) { background: transparent; font-size: 12.5px; }
.scm-list :deep(.ant-tree .ant-tree-node-content-wrapper) { display: inline-flex; align-items: center; min-width: 0; flex: 1; padding: 0; }
.scm-list :deep(.ant-tree .ant-tree-node-content-wrapper:hover) { background: transparent; }
.scm-list :deep(.ant-tree .ant-tree-treenode) { padding: 0; align-items: center; }
.scm-list :deep(.ant-tree .ant-tree-switcher) { width: 18px; display: flex; align-items: center; justify-content: center; }

/* 提交框 */
.scm-foot { flex-shrink: 0; padding: 10px; border-top: 1px solid var(--color-border-light, #f0f0ee); background: var(--editor-panel-bg, #fff); }
</style>
