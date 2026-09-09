<template>
  <!-- 停靠式右侧 AI 栏;两种模式:对话 / 编辑代理 -->
  <div v-show="open" class="ai-dock" :style="{ width: width + 'px' }">
    <div class="ai-resize-handle" @mousedown="startResize"></div>

    <div class="ai-dock-head">
      <div class="ai-dock-title">
        <AiIcon :size="16" class="ai-title-icon" />
        <a-radio-group v-model:value="mode" size="small" button-style="solid" :disabled="busy">
          <a-radio-button value="chat">对话</a-radio-button>
          <a-radio-button value="agent">编辑代理</a-radio-button>
        </a-radio-group>
      </div>
      <div class="ai-dock-head-right">
        <span v-if="mode === 'agent' && totalTokens > 0" class="ai-token-meter" :title="`累计 token(入 ${tokIn}/出 ${tokOut})`">
          {{ (totalTokens / 1000).toFixed(1) }}k tok
        </span>
        <a-button v-if="mode === 'agent' && !busy && timeline.length > 0" type="text" size="small" @click="resetAgent">重置</a-button>
        <a-button v-if="mode === 'chat' && !loading" type="text" size="small" @click="clearChat">清空</a-button>
        <a-button type="text" size="small" @click="open = false">
          <template #icon><CloseOutlined /></template>
        </a-button>
      </div>
    </div>

    <!-- ===== 对话模式 ===== -->
    <template v-if="mode === 'chat'">
      <div class="ai-body">
        <Welcome v-if="messages.length === 0" class="ai-welcome"
          :icon="() => h('span', { class: 'ai-welcome-icon' }, [h(AiIcon, { size: 36 })])"
          title="AI 助手" description="问我关于当前模板、变量或代码的问题" />
        <BubbleList v-else :items="chatItems" :roles="chatRoles" class="ai-bubble-list" />
        <div v-if="loading" class="ai-loading-row"><Bubble :loading="true" /></div>
      </div>
      <div class="ai-sender-wrap">
        <Sender v-model:value="input" :loading="loading" placeholder="输入问题,Enter 发送"
          submit-type="enter" @submit="send" />
      </div>
    </template>

    <!-- ===== 编辑代理模式 ===== -->
    <template v-else>
      <div class="ai-body">
        <Welcome v-if="timeline.length === 0" class="ai-welcome"
          :icon="() => h('span', { class: 'ai-welcome-icon' }, [h(AiIcon, { size: 36 })])"
          title="编辑代理" description="给 AI 一个编辑任务,它会读取文件、打补丁、渲染验证" />
        <div v-else class="agent-stream">
          <!-- 紧凑步骤行(终端式:单行折叠,点击展开) -->
          <div class="steps">
            <div v-for="(e, i) in timeline" :key="i" class="step" :class="e.kind" @click="toggleStep(i)">
              <span class="step-dot"></span>
              <span class="step-title">{{ e.title }}</span>
              <span v-if="e.detail" class="step-detail">{{ e.detail }}</span>
              <span class="step-chev">›</span>
            </div>
            <div v-if="expandedStep !== null && timeline[expandedStep]?.full" class="step-full">{{ timeline[expandedStep].full }}</div>
          </div>

          <!-- diff 卡片 -->
          <div v-for="(d, i) in dirtyFiles" :key="'d' + i" class="diff-card">
            <div class="diff-head">
              <span class="diff-path">{{ d.path }}</span>
              <span class="diff-count">+{{ d.added }} / -{{ d.removed }}</span>
            </div>
            <pre class="diff-body">{{ d.preview }}</pre>
          </div>

          <Bubble v-if="agentSummary" :content="agentSummary" class="ai-summary" />
        </div>
      </div>

      <div class="ai-sender-wrap">
        <div v-if="dirtyFiles.length > 0 && !busy" class="ai-apply-row">
          <a-button type="primary" size="small" :loading="applying" @click="applyAll">应用全部修改({{ dirtyFiles.length }})</a-button>
          <a-button size="small" :disabled="applying" @click="discardAll">全部放弃</a-button>
        </div>
        <Sender v-model:value="agentInput" :loading="agentRunning" :disabled="applying"
          placeholder="描述编辑任务,如:把端口 8080 提取为变量"
          submit-type="enter" @submit="runAgent" @cancel="abortAgent" />
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, reactive, computed, onMounted, onUnmounted, h } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { CloseOutlined } from '@ant-design/icons-vue'
import { Bubble, BubbleList, Sender, Welcome } from 'ant-design-x-vue'
import AiIcon from '@/components/icons/AiIcon.vue'
import { getTemplateFileTree, getTemplateFileContent, editTemplateFile, addTemplateFile } from '@/api/editor/templateFiles'

const props = defineProps({
  currentFilePath: { type: String, default: '' },
  currentFileContent: { type: String, default: '' },
  templateVariables: { type: Array, default: () => [] },
  templateId: { type: [String, Number], required: true },
})

const emit = defineEmits(['buffer-replace', 'files-updated'])

// 路由参数是字符串,服务端 body 需要 i64
const tid = () => Number(props.templateId)
const open = defineModel('open', { type: Boolean, default: false })

// ---- 布局:宽度拖拽 ----
const WIDTH_KEY = 'editor_ai_dock_width'
const MIN_W = 320
const MAX_W = 640
const width = ref(Number(localStorage.getItem(WIDTH_KEY)) || 420)
let dragging = false
let startX = 0
let startW = 0
const onDragMove = (e) => {
  if (!dragging) return
  width.value = Math.min(MAX_W, Math.max(MIN_W, startW + (startX - e.clientX)))
}
const onDragEnd = () => {
  if (!dragging) return
  dragging = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
  localStorage.setItem(WIDTH_KEY, String(width.value))
}
const startResize = (e) => {
  dragging = true
  startX = e.clientX
  startW = width.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  e.preventDefault()
}
onMounted(() => {
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
})
onUnmounted(() => {
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
})

// ===== 对话模式 =====
const mode = ref('chat')
const input = ref('')
const loading = ref(false)
const messages = ref([])

const chatItems = computed(() =>
  messages.value.map((m, i) => ({ key: String(i), role: m.role, content: m.content }))
)
const chatRoles = {
  user: { placement: 'end', variant: 'filled' },
  assistant: { placement: 'start' },
}

const buildExtraContext = () => {
  const parts = []
  if (props.currentFilePath) parts.push(`当前打开的文件: ${props.currentFilePath}`)
  const names = props.templateVariables.map((v) => v.fieldName || v.name).filter(Boolean).slice(0, 40)
  if (names.length > 0) parts.push(`模板已有变量: ${names.join(', ')}`)
  return parts.join('\n')
}

const send = async (textArg) => {
  const text = (typeof textArg === 'string' ? textArg : input.value).trim()
  if (!text || loading.value) return
  messages.value.push({ role: 'user', content: text })
  input.value = ''
  loading.value = true
  try {
    const history = messages.value.slice(0, -1).slice(-20).map((m) => ({ role: m.role, content: m.content }))
    const result = await invoke('ai_chat', {
      message: text, templatePath: null, projectId: null,
      extraContext: buildExtraContext(), history,
    })
    const data = JSON.parse(result)
    messages.value.push({ role: 'assistant', content: data.response || '抱歉,没有拿到有效回复。' })
  } catch (e) {
    messages.value.push({ role: 'assistant', content: `调用失败: ${e.message || e}` })
  } finally {
    loading.value = false
  }
}
const clearChat = () => { messages.value = [] }

// ===== 编辑代理模式 =====
const agentInput = ref('')
const agentRunning = ref(false)
const applying = ref(false)
const timeline = ref([])
const agentSummary = ref('')
const abortFlag = ref(false)
const taskMessages = ref([])

// ===== 会话持久化(localStorage,按模板隔离;续跑时工具结果已裁剪) =====
let sessionName = null  // 当前会话时间戳名;null=新会话
const saveTimer = { t: null }
const scheduleSave = () => {
  clearTimeout(saveTimer.t)
  saveTimer.t = setTimeout(saveSession, 500)
}
const saveSession = async () => {
  try {
    const trimmed = taskMessages.value.map((m) =>
      m.role === 'tool_result'
        ? { ...m, content: String(m.content || '').slice(0, 2000) }
        : m
    )
    if (!sessionName) sessionName = String(Date.now())
    const line = (o) => JSON.stringify(o)
    const lines = [
      line({ t: 'meta', tokIn: tokIn.value, tokOut: tokOut.value, savedAt: Date.now() }),
      ...messages.value.map((m) => line({ t: 'chat', m })),
      ...timeline.value.map((e) => line({ t: 'tl', e: { ...e, full: e.full ? String(e.full).slice(0, 4000) : undefined } })),
      ...trimmed.map((m) => line({ t: 'task', m })),
      ...[...workset.entries()].map(([path, f]) => line({ t: 'file', path, f: { ...f } })),
    ]
    await invoke('ai_session_save', {
      templateId: Number(props.templateId),
      name: sessionName,
      data: lines.join('\n'),
    })
  } catch { /* 写盘失败静默,内存态继续 */ }
}
const loadSession = async () => {
  try {
    const raw = await invoke('ai_session_load', { templateId: Number(props.templateId) })
    if (!raw || !raw.data) return
    sessionName = raw.name.replace('.jsonl', '')
    const d = { chat: [], tl: [], task: [], file: [], meta: {} }
    for (const l of raw.data.split('\n')) {
      if (!l.trim()) continue
      try { const o = JSON.parse(l); (d[o.t] || (d[o.t] = [])).push(o.m ?? o.e ?? o.f ?? o) } catch {}
    }
    const meta = (d.meta && d.meta[0]) || {}
    messages.value = d.chat || []
    timeline.value = d.tl || []
    taskMessages.value = d.task || []
    tokIn.value = meta.tokIn || 0
    tokOut.value = meta.tokOut || 0
    agentSummary.value = ''
    workset.clear()
    for (const item of d.file || []) {
      if (item && item.path) workset.set(item.path, { ...item.f, version: 0, readVersion: 0, hasRead: true })
    }
    refreshDirty()
  } catch { /* 损坏则丢弃 */ }
}
const clearSession = () => { try { if (sessionName) invoke('ai_session_clear', { templateId: Number(props.templateId), name: sessionName + '.jsonl' }); sessionName = null } catch {} }
const busy = computed(() => agentRunning.value || applying.value)

// token 水位
const tokIn = ref(0)
const tokOut = ref(0)
const totalTokens = computed(() => tokIn.value + tokOut.value)

// 工作副本:path → {content, base, version, readVersion, hasRead}
const workset = reactive(new Map())
let lastSeenBuffer = ''
watch(() => [props.currentFilePath, props.currentFileContent], ([p, c]) => {
  if (p && workset.has(p) && c !== lastSeenBuffer) {
    const f = workset.get(p)
    if (f.content !== c) f.version += 1
  }
  lastSeenBuffer = c
})

const ensureLoaded = async (path) => {
  if (workset.has(path)) return workset.get(path)
  let content = ''
  if (path === props.currentFilePath) {
    content = props.currentFileContent || ''
  } else {
    const res = await getTemplateFileContent(tid(), path)
    content = res.data?.data?.content ?? res.data?.data?.fileContent ?? ''
  }
  const f = { content, base: content, version: 0, readVersion: -1, hasRead: false }
  workset.set(path, f)
  return f
}

// ---- 工具 schema ----
const str = (desc) => ({ type: 'string', description: desc })
const TOOLS = [
  { name: 'list_files', description: '列出模板的全部文件路径', parameters: { type: 'object', properties: {}, required: [] } },
  { name: 'read_file', description: '读取一个文件的当前内容(编辑前必须先读)', parameters: { type: 'object', properties: { path: str('文件相对路径') }, required: ['path'] } },
  { name: 'edit_file', description: '精确匹配替换:old_string 必须与文件内容完全一致且唯一', parameters: { type: 'object', properties: { path: str('文件相对路径'), old_string: str('要替换的原文本(含缩进,须唯一)'), new_string: str('替换后的文本') }, required: ['path', 'old_string', 'new_string'] } },
  { name: 'insert_lines', description: '在指定行号前插入文本(1 起;超过文件长度则追加到末尾)', parameters: { type: 'object', properties: { path: str('文件相对路径'), line: { type: 'integer', description: '插入位置的行号' }, content: str('插入的文本') }, required: ['path', 'line', 'content'] } },
  { name: 'create_file', description: '创建新文件(全量内容)', parameters: { type: 'object', properties: { path: str('新文件相对路径'), content: str('文件全量内容') }, required: ['path', 'content'] } },
  { name: 'render_file', description: '按工作副本当前内容本地渲染验证语法', parameters: { type: 'object', properties: { path: str('文件相对路径') }, required: ['path'] } },
  { name: 'list_variables', description: '列出模板已定义的变量', parameters: { type: 'object', properties: {}, required: [] } },
]

// ---- 工具执行(读前置/新鲜度守卫) ----
async function execTool(name, args) {
  switch (name) {
    case 'list_files': {
      const res = await getTemplateFileTree(tid())
      const tree = res.data?.data?.tree || []
      const paths = []
      const walk = (nodes) => nodes.forEach((n) => {
        if (n.isDirectory || n.is_directory) { walk(n.children || []) } else { paths.push(n.filePath || n.file_path) }
      })
      walk(tree)
      return paths.length ? paths.join('\n') : '(空模板)'
    }
    case 'read_file': {
      const f = await ensureLoaded(args.path)
      f.readVersion = f.version
      f.hasRead = true
      const c = f.content || ''
      return c.length > 24000 ? c.slice(0, 24000) + '\n...(截断)' : (c || '(空文件)')
    }
    case 'edit_file': {
      const f = await ensureLoaded(args.path)
      if (!f.hasRead) return '错误:必须先 read_file 读取该文件才能编辑。'
      if (f.readVersion !== f.version) return '错误:文件在你读取后被修改,请重新 read_file。'
      const first = f.content.indexOf(args.old_string)
      if (first === -1) return '错误:old_string 未在文件中找到(注意缩进与完全一致),请重读后重试。'
      if (f.content.indexOf(args.old_string, first + 1) !== -1) return '错误:old_string 在文件中出现多次,请扩大范围使其唯一。'
      f.content = f.content.slice(0, first) + args.new_string + f.content.slice(first + args.old_string.length)
      f.version += 1
      f.readVersion = f.version
      return '已应用替换。'
    }
    case 'insert_lines': {
      const f = await ensureLoaded(args.path)
      if (!f.hasRead) return '错误:必须先 read_file 读取该文件才能编辑。'
      if (f.readVersion !== f.version) return '错误:文件在你读取后被修改,请重新 read_file。'
      const lines = f.content.split('\n')
      const at = Math.max(0, Math.min(args.line - 1, lines.length))
      lines.splice(at, 0, ...args.content.split('\n'))
      f.content = lines.join('\n')
      f.version += 1
      f.readVersion = f.version
      return '已插入。'
    }
    case 'create_file': {
      if (workset.has(args.path)) return '错误:文件已存在,请用 edit_file。'
      workset.set(args.path, { content: args.content, base: '', version: 0, readVersion: 0, hasRead: true })
      return '已创建(待应用)。'
    }
    case 'render_file': {
      const f = await ensureLoaded(args.path)
      const vars = {}
      props.templateVariables.forEach((v) => {
        const n = v.fieldName || v.name
        if (n) vars[n] = v.defaultValue ?? v.value ?? 'test'
      })
      const r = await invoke('render_string_content', { template: f.content, variables: vars })
      if (r && r.success) return `渲染成功(${r.content?.length ?? 0} 字符)`
      return `渲染失败: ${r?.error?.message || r?.error?.type || '未知错误'}`
    }
    case 'list_variables': {
      const names = props.templateVariables.map((v) => v.fieldName || v.name).filter(Boolean)
      return names.length ? names.join(', ') : '(无变量)'
    }
    default:
      return `错误:未知工具 ${name}`
  }
}

// ---- 紧凑步骤行展开态 ----
const expandedStep = ref(null)
const toggleStep = (i) => {
  expandedStep.value = expandedStep.value === i ? null : i
}

// ---- diff 视图(公共前后缀行级差异) ----
const dirtyFiles = ref([])
const refreshDirty = () => {
  const list = []
  for (const [path, f] of workset) {
    if (path === props.currentFilePath && f.content === (props.currentFileContent || '')) continue
    const orig = (f.base || '').split('\n')
    const now = (f.content || '').split('\n')
    let s = 0
    while (s < orig.length && s < now.length && orig[s] === now[s]) s += 1
    let e = 0
    while (e < orig.length - s && e < now.length - s && orig[orig.length - 1 - e] === now[now.length - 1 - e]) e += 1
    const removed = orig.length - s - e
    const added = now.length - s - e
    if (removed === 0 && added === 0) continue
    const preview = now.slice(s, s + Math.max(added, Math.min(removed, 4), 1)).slice(0, 12).join('\n')
    list.push({ path, added, removed, preview })
  }
  dirtyFiles.value = list
}
watch(workset, () => nextTick(refreshDirty), { deep: true })

const abortAgent = () => {
  abortFlag.value = true
  timeline.value.push({ kind: 'error', title: '用户中止' })
}

const runAgent = async (textArg) => {
  const task = (typeof textArg === 'string' ? textArg : agentInput.value).trim()
  if (!task || agentRunning.value) return
  agentInput.value = ''
  agentRunning.value = true
  abortFlag.value = false
  timeline.value.push({ kind: 'tool', title: '任务', detail: task })

  // 会话续跑:已有线程则追加任务,否则以系统提示开局
  if (taskMessages.value.length === 0) {
    let system = ''
    try {
      system = await invoke('ai_get_agent_prompt')
    } catch { system = '' }
    const ctx = buildExtraContext()
    taskMessages.value = [
      { role: 'system', content: (system || '') + (ctx ? `\n\n当前编辑上下文:\n${ctx}` : '') },
    ]
  }
  taskMessages.value.push({ role: 'user', content: task })

  try {
    for (let round = 0; round < 12; round++) {
      if (abortFlag.value) break
      const raw = await invoke('ai_agent_turn', { messages: taskMessages.value, tools: TOOLS })
      const res = JSON.parse(raw)
      if (res.usage) {
        tokIn.value += res.usage.input || 0
        tokOut.value += res.usage.output || 0
      }
      if (res.type === 'final') {
        agentSummary.value = res.text || '(无总结)'
        timeline.value.push({ kind: 'done', title: '完成' })
        break
      }
      taskMessages.value.push({ role: 'assistant', tool_calls: res.calls })
      for (const c of res.calls) {
        if (abortFlag.value) break
        let out
        try {
          out = await execTool(c.name, c.arguments || {})
        } catch (e) {
          out = `工具执行异常: ${e.message || e}`
        }
        taskMessages.value.push({ role: 'tool_result', tool_call_id: c.id, name: c.name, content: String(out).slice(0, 8000) })
        timeline.value.push({
          kind: out.startsWith('错误') ? 'error' : 'tool',
          title: `${c.name}(${(c.arguments?.path || '').slice(0, 40)})`,
          detail: String(out).split('\n')[0].slice(0, 120),
        })
      }
    }
    refreshDirty()
  } catch (e) {
    timeline.value.push({ kind: 'error', title: '中止', detail: e.message || String(e) })
  } finally {
    agentRunning.value = false
    abortFlag.value = false
  }
}

// ---- 应用 / 放弃 ----
const fileExistsCache = new Set()
const fileExistsOnServer = async (path) => {
  if (fileExistsCache.has(path)) return true
  try {
    await getTemplateFileContent(tid(), path)
    fileExistsCache.add(path)
    return true
  } catch {
    return false
  }
}

const applyAll = async () => {
  applying.value = true
  try {
    let applied = 0
    for (const d of dirtyFiles.value) {
      const f = workset.get(d.path)
      if (!(await fileExistsOnServer(d.path))) {
        const parentPath = d.path.includes('/') ? d.path.slice(0, d.path.lastIndexOf('/')) : ''
        await addTemplateFile({ templateId: tid(), fileName: d.path.split('/').pop(), parentPath, isDirectory: false })
        fileExistsCache.add(d.path)
      }
      await editTemplateFile({ templateId: tid(), filePath: d.path, content: f.content })
      applied += 1
    }
    message.success(`已应用 ${applied} 个文件`)
    for (const d of dirtyFiles.value) {
      if (d.path === props.currentFilePath) emit('buffer-replace', { path: d.path, content: workset.get(d.path).content })
    }
    emit('files-updated')
    workset.clear()
    dirtyFiles.value = []
    timeline.value.push({ kind: 'done', title: `已应用 ${applied} 个文件` })
  } catch (e) {
    message.error('应用失败: ' + (e.message || e))
  } finally {
    applying.value = false
  }
}

const discardAll = () => {
  workset.clear()
  dirtyFiles.value = []
  timeline.value.push({ kind: 'done', title: '已放弃全部未应用修改' })
}

const resetAgent = () => {
  clearSession()
  taskMessages.value = []
  workset.clear()
  dirtyFiles.value = []
  timeline.value = []
  agentSummary.value = ''
  tokIn.value = 0
  tokOut.value = 0
}
// 状态全部声明后恢复会话并挂自动保存(避免 TDZ)
loadSession()
const persistWatch = watch(
  [messages, timeline, agentSummary, tokIn, tokOut, workset, taskMessages],
  scheduleSave,
  { deep: true }
)
onUnmounted(() => { clearTimeout(saveTimer.t); persistWatch.stop() })

</script>

<style scoped>
.ai-dock { position: relative; height: 100%; flex-shrink: 0; display: flex; flex-direction: column; background: var(--editor-panel-bg, #fff); border-left: 1px solid var(--editor-border, #e0e0e6); overflow: hidden; }
.ai-resize-handle { position: absolute; left: 0; top: 0; width: 5px; height: 100%; cursor: col-resize; z-index: 10; transition: background-color 0.15s ease; }
.ai-resize-handle:hover { background: var(--editor-accent, #16a34a); }
.ai-dock-head { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px 10px 16px; border-bottom: 1px solid var(--editor-border, #e0e0e6); flex-shrink: 0; }
.ai-dock-title { display: flex; align-items: center; gap: 10px; font-size: 14px; font-weight: 600; color: var(--editor-primary, #1b1c1f); }
.ai-title-icon { color: var(--editor-accent, #16a34a); }
.ai-dock-head-right { display: flex; align-items: center; gap: 4px; }
.ai-token-meter { font-size: 11.5px; color: var(--editor-muted, #999); padding: 0 6px; cursor: default; }

.ai-body { flex: 1; min-height: 0; display: flex; flex-direction: column; overflow: hidden; }
.ai-bubble-list { flex: 1; min-height: 0; padding: 12px 10px; }
.agent-stream { flex: 1; min-height: 0; overflow-y: auto; padding: 14px 14px 10px; display: flex; flex-direction: column; gap: 12px; }
.ai-loading-row { padding: 0 14px 10px; }
.ai-welcome { flex: 1; justify-content: center; }
.ai-welcome-icon { color: var(--editor-accent, #16a34a); opacity: 0.7; display: inline-flex; }

.ai-sender-wrap { display: flex; flex-direction: column; gap: 8px; padding: 10px 12px 12px; border-top: 1px solid var(--editor-border, #e0e0e6); flex-shrink: 0; }
.ai-apply-row { display: flex; gap: 8px; }

.diff-card { border: 1px solid var(--editor-border, #e0e0e6); border-radius: 8px; overflow: hidden; }
.diff-head { display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--editor-inset-bg, #f4f4f2); font-size: 12px; }
.diff-path { font-weight: 500; color: var(--editor-primary, #333); word-break: break-all; }
.diff-count { color: var(--editor-accent, #16a34a); flex: none; }
.diff-body { margin: 0; padding: 8px 10px; font-size: 11px; line-height: 1.5; max-height: 120px; overflow: auto; color: var(--editor-muted, #666); white-space: pre-wrap; word-break: break-all; }
</style>
