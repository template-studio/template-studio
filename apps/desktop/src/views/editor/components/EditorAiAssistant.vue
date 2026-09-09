<template>
  <!-- 停靠式右侧 AI 栏;两种模式:对话 / 编辑代理 -->
  <div v-show="open" class="ai-dock" :style="{ width: width + 'px' }">
    <div class="ai-resize-handle" @mousedown="startResize"></div>

    <div class="ai-dock-head">
      <div class="ai-dock-title">
        <AiIcon :size="16" class="ai-title-icon" />
        <a-radio-group v-model:value="mode" size="small" button-style="solid" :disabled="agentRunning">
          <a-radio-button value="chat">对话</a-radio-button>
          <a-radio-button value="agent">编辑代理</a-radio-button>
        </a-radio-group>
      </div>
      <div class="ai-dock-actions">
        <a-button v-if="mode === 'agent'" type="text" size="small" :disabled="agentRunning" @click="resetAgent">重置</a-button>
        <a-button v-if="mode === 'chat'" type="text" size="small" @click="clearChat">清空</a-button>
        <a-button type="text" size="small" @click="open = false">
          <template #icon><CloseOutlined /></template>
        </a-button>
      </div>
    </div>

    <!-- ===== 对话模式 ===== -->
    <template v-if="mode === 'chat'">
      <div ref="messagesRef" class="ai-messages">
        <div v-if="messages.length === 0" class="ai-empty">
          <AiIcon :size="40" class="ai-empty-icon" />
          <p>问我关于当前模板、变量或代码的问题</p>
          <div class="ai-hints">
            <button v-for="h in hints" :key="h" class="ai-hint-chip" @click="send(h)">{{ h }}</button>
          </div>
        </div>
        <div v-for="(m, i) in messages" :key="i" class="ai-msg" :class="m.role">
          <div class="ai-bubble">{{ m.content }}</div>
        </div>
        <div v-if="loading" class="ai-msg assistant">
          <div class="ai-bubble ai-typing"><span></span><span></span><span></span></div>
        </div>
      </div>
      <div class="ai-input-row">
        <a-textarea v-model:value="input" placeholder="输入问题,Enter 发送,Shift+Enter 换行"
          :auto-size="{ minRows: 2, maxRows: 6 }" :disabled="loading"
          @keydown.enter.exact.prevent="send()" />
        <a-button type="primary" :loading="loading" :disabled="!input.trim()" class="ai-send-btn" @click="send()">
          <template #icon><SendOutlined /></template><span>发送</span>
        </a-button>
      </div>
    </template>

    <!-- ===== 编辑代理模式 ===== -->
    <template v-else>
      <div class="ai-messages">
        <div v-if="timeline.length === 0" class="ai-empty">
          <AiIcon :size="40" class="ai-empty-icon" />
          <p>给 AI 一个编辑任务,它会读取文件、打补丁、渲染验证</p>
          <div class="ai-hints">
            <button v-for="h in agentHints" :key="h" class="ai-hint-chip" @click="runAgent(h)">{{ h }}</button>
          </div>
        </div>

        <!-- 工具时间线 -->
        <div v-for="(e, i) in timeline" :key="i" class="tl-item" :class="e.kind">
          <span class="tl-icon">{{ e.kind === 'tool' ? '⚙' : e.kind === 'error' ? '✕' : '✓' }}</span>
          <div class="tl-body">
            <div class="tl-title">{{ e.title }}</div>
            <div v-if="e.detail" class="tl-detail">{{ e.detail }}</div>
          </div>
        </div>

        <!-- diff 卡片 -->
        <div v-for="(d, i) in dirtyFiles" :key="'d' + i" class="diff-card">
          <div class="diff-head">
            <span class="diff-path">{{ d.path }}</span>
            <span class="diff-count">+{{ d.added }} / -{{ d.removed }}</span>
          </div>
          <pre class="diff-body">{{ d.preview }}</pre>
        </div>

        <div v-if="agentSummary" class="ai-msg assistant">
          <div class="ai-bubble">{{ agentSummary }}</div>
        </div>
      </div>

      <div class="ai-input-row">
        <div v-if="dirtyFiles.length > 0 && !agentRunning" class="ai-apply-row">
          <a-button type="primary" size="small" :loading="applying" @click="applyAll">应用全部修改({{ dirtyFiles.length }})</a-button>
          <a-button size="small" :disabled="applying" @click="discardAll">全部放弃</a-button>
        </div>
        <div class="ai-input-flex">
          <a-textarea v-model:value="agentInput" :placeholder="agentRunning ? '代理执行中…' : '描述编辑任务,如:把端口 8080 改成变量 {{ server_port }}'"
            :auto-size="{ minRows: 2, maxRows: 6 }" :disabled="agentRunning"
            @keydown.enter.exact.prevent="runAgent()" />
          <a-button type="primary" :loading="agentRunning" :disabled="!agentInput.trim() && !agentRunning" @click="runAgent()">
            <template #icon><SendOutlined /></template><span>执行</span>
          </a-button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, reactive, onMounted, onUnmounted } from 'vue'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { CloseOutlined, SendOutlined } from '@ant-design/icons-vue'
import AiIcon from '@/components/icons/AiIcon.vue'
import { getTemplateFileTree, getTemplateFileContent, editTemplateFile, addTemplateFile } from '@/api/editor/templateFiles'

const props = defineProps({
  currentFilePath: { type: String, default: '' },
  currentFileContent: { type: String, default: '' },
  templateVariables: { type: Array, default: () => [] },
  templateId: { type: [String, Number], required: true },
})

const emit = defineEmits(['buffer-replace', 'files-updated'])
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

// ===== 对话模式(与此前一致) =====
const mode = ref('chat')
const input = ref('')
const loading = ref(false)
const messages = ref([])
const messagesRef = ref()
const hints = ['这个模板还缺什么变量?', '帮我优化当前文件的模板语法', '变量命名有什么建议?']

watch(messages, () => {
  nextTick(() => { if (messagesRef.value) messagesRef.value.scrollTop = messagesRef.value.scrollHeight })
}, { deep: true })

const buildExtraContext = () => {
  const parts = []
  if (props.currentFilePath) parts.push(`当前打开的文件: ${props.currentFilePath}`)
  const names = props.templateVariables.map((v) => v.fieldName || v.name).filter(Boolean).slice(0, 40)
  if (names.length > 0) parts.push(`模板已有变量: ${names.join(', ')}`)
  return parts.join('\n')
}

const send = async (preset) => {
  const text = (preset || input.value).trim()
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
const agentHints = [
  '把硬编码的端口号提取为变量',
  '给所有 SQL 文件加上统一文件头注释',
  '检查并修复模板语法错误',
]

// 工作副本:path → {content, version, readVersion, hasRead}
const workset = reactive(new Map())
let lastSeenBuffer = ''
watch(() => [props.currentFilePath, props.currentFileContent], ([p, c]) => {
  // 打开文件的内容变化(用户编辑)→ bump 版本,触发 AI 侧过期
  if (p && workset.has(p) && c !== lastSeenBuffer) {
    const f = workset.get(p)
    if (f.content !== c) f.version += 1
  }
  lastSeenBuffer = c
})

const wcGet = (path) => workset.get(path)
const ensureLoaded = async (path) => {
  if (workset.has(path)) return workset.get(path)
  let content = ''
  if (path === props.currentFilePath) {
    content = props.currentFileContent || ''
  } else {
    const res = await getTemplateFileContent(props.templateId, path)
    content = res.data?.data?.content ?? res.data?.data?.fileContent ?? ''
  }
  const f = { content, base: content, version: 0, readVersion: -1, hasRead: false }
  workset.set(path, f)
  return f
}

// ---- 工具 schema(给模型) ----
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

// ---- 工具执行(含读前置/新鲜度守卫) ----
async function execTool(name, args) {
  switch (name) {
    case 'list_files': {
      const res = await getTemplateFileTree(props.templateId)
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

// ---- diff 视图(朴素行级差异:公共前后缀) ----
const dirtyFiles = ref([])
const refreshDirty = () => {
  const list = []
  for (const [path, f] of workset) {
    if (path === props.currentFilePath && f.content === (props.currentFileContent || '')) continue
    const orig = path === props.currentFilePath ? (props.currentFileContent || '').split('\n')
      : f.hasRead ? [] : []
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

const runAgent = async (preset) => {
  const task = (preset || agentInput.value).trim()
  if (!task || agentRunning.value) return
  agentInput.value = ''
  agentRunning.value = true
  agentSummary.value = ''
  timeline.value = [{ kind: 'tool', title: '任务', detail: task }]

  // 系统提示词:Rust 资产 + 当前编辑上下文
  let system = ''
  try {
    const r = await invoke('ai_get_agent_prompt')
    system = r || ''
  } catch { system = '' }
  const ctx = buildExtraContext()
  const taskMessages = [
    { role: 'system', content: system + (ctx ? `\n\n当前编辑上下文:\n${ctx}` : '') },
    { role: 'user', content: task },
  ]

  try {
    for (let round = 0; round < 12; round++) {
      const raw = await invoke('ai_agent_turn', { messages: taskMessages, tools: TOOLS })
      const res = JSON.parse(raw)
      if (res.type === 'final') {
        agentSummary.value = res.text || '(无总结)'
        timeline.value.push({ kind: 'done', title: '完成' })
        break
      }
      taskMessages.push({ role: 'assistant', tool_calls: res.calls })
      for (const c of res.calls) {
        let out
        try {
          out = await execTool(c.name, c.arguments || {})
        } catch (e) {
          out = `工具执行异常: ${e.message || e}`
        }
        taskMessages.push({ role: 'tool_result', tool_call_id: c.id, name: c.name, content: String(out).slice(0, 8000) })
        const brief = String(out).split('\n')[0].slice(0, 120)
        timeline.value.push({
          kind: out.startsWith('错误') ? 'error' : 'tool',
          title: `${c.name}(${(c.arguments?.path || '').slice(0, 40)})`,
          detail: brief,
        })
      }
    }
    refreshDirty()
  } catch (e) {
    timeline.value.push({ kind: 'error', title: '中止', detail: e.message || String(e) })
  } finally {
    agentRunning.value = false
  }
}

// ---- 应用 / 放弃(走既有保存通道 editTemplateFile) ----
const applyAll = async () => {
  applying.value = true
  try {
    let applied = 0
    for (const d of dirtyFiles.value) {
      const f = wcGet(d.path)
      const existsBefore = d.path !== props.currentFilePath && !(await fileExistsOnServer(d.path))
      if (existsBefore === false && !(f && f.hasRead)) {
        // 新文件:先建条目
        const parentPath = d.path.includes('/') ? d.path.slice(0, d.path.lastIndexOf('/')) : ''
        await addTemplateFile({ templateId: props.templateId, fileName: d.path.split('/').pop(), parentPath, isDirectory: false })
      }
      await editTemplateFile({ templateId: props.templateId, filePath: d.path, content: f.content })
      applied += 1
    }
    message.success(`已应用 ${applied} 个文件`)
    // 打开中的文件直接换缓冲区,其余刷新文件树
    for (const d of dirtyFiles.value) {
      if (d.path === props.currentFilePath) emit('buffer-replace', { path: d.path, content: wcGet(d.path).content })
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

const fileExistsCache = new Set()
const fileExistsOnServer = async (path) => {
  if (fileExistsCache.has(path)) return true
  try {
    await getTemplateFileContent(props.templateId, path)
    fileExistsCache.add(path)
    return true
  } catch {
    return false
  }
}

const discardAll = () => {
  workset.clear()
  dirtyFiles.value = []
  timeline.value.push({ kind: 'done', title: '已放弃全部未应用修改' })
}

const resetAgent = () => {
  workset.clear()
  dirtyFiles.value = []
  timeline.value = []
  agentSummary.value = ''
}
</script>

<style scoped>
.ai-dock { position: relative; height: 100%; flex-shrink: 0; display: flex; flex-direction: column; background: var(--editor-panel-bg, #fff); border-left: 1px solid var(--editor-border, #e0e0e6); overflow: hidden; }
.ai-resize-handle { position: absolute; left: 0; top: 0; width: 5px; height: 100%; cursor: col-resize; z-index: 10; transition: background-color 0.15s ease; }
.ai-resize-handle:hover { background: var(--editor-accent, #16a34a); }
.ai-dock-head { display: flex; align-items: center; justify-content: space-between; padding: 10px 12px 10px 16px; border-bottom: 1px solid var(--editor-border, #e0e0e6); flex-shrink: 0; }
.ai-dock-title { display: flex; align-items: center; gap: 10px; font-size: 14px; font-weight: 600; color: var(--editor-primary, #1b1c1f); }
.ai-title-icon { color: var(--editor-accent, #16a34a); }
.ai-dock-actions { display: flex; gap: 4px; }
.ai-messages { flex: 1; overflow-y: auto; padding: 16px 14px 12px; display: flex; flex-direction: column; gap: 12px; }
.ai-empty { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; color: var(--editor-muted, #999); }
.ai-empty-icon { opacity: 0.45; }
.ai-empty p { margin: 0; font-size: 13px; }
.ai-hints { display: flex; flex-direction: column; gap: 8px; align-items: stretch; width: 88%; }
.ai-hint-chip { border: 1px solid var(--editor-border, #e0e0e6); background: var(--editor-inset-bg, #f8f9fa); color: var(--editor-primary, #333); border-radius: 8px; padding: 8px 14px; font-size: 13px; cursor: pointer; text-align: left; transition: border-color 0.15s ease; }
.ai-hint-chip:hover { border-color: var(--editor-accent, #16a34a); }
.ai-msg { display: flex; }
.ai-msg.user { justify-content: flex-end; }
.ai-bubble { max-width: 86%; padding: 10px 14px; border-radius: 12px; font-size: 13.5px; line-height: 1.65; white-space: pre-wrap; word-break: break-word; }
.ai-msg.user .ai-bubble { background: var(--editor-primary, #1b1c1f); color: var(--editor-bg, #fff); border-bottom-right-radius: 4px; }
.ai-msg.assistant .ai-bubble { background: var(--editor-inset-bg, #f4f4f2); color: var(--editor-primary, #333); border-bottom-left-radius: 4px; }
.ai-typing { display: flex; gap: 4px; align-items: center; padding: 14px 16px; }
.ai-typing span { width: 6px; height: 6px; border-radius: 50%; background: var(--editor-muted, #999); animation: ai-dot 1.2s infinite ease-in-out; }
.ai-typing span:nth-child(2) { animation-delay: 0.15s; }
.ai-typing span:nth-child(3) { animation-delay: 0.3s; }
@keyframes ai-dot { 0%, 60%, 100% { transform: translateY(0); opacity: 0.4; } 30% { transform: translateY(-4px); opacity: 1; } }
.ai-input-row { display: flex; flex-direction: column; gap: 8px; padding: 12px 14px 14px; border-top: 1px solid var(--editor-border, #e0e0e6); flex-shrink: 0; }
.ai-input-flex { display: flex; flex-direction: column; gap: 8px; }
.ai-send-btn { align-self: flex-end; }

/* agent 时间线与 diff */
.tl-item { display: flex; gap: 8px; font-size: 12.5px; align-items: flex-start; }
.tl-icon { flex: none; width: 18px; height: 18px; border-radius: 5px; display: flex; align-items: center; justify-content: center; font-size: 11px; background: var(--editor-inset-bg, #f4f4f2); color: var(--editor-muted, #999); }
.tl-item.error .tl-icon { color: #e5484d; }
.tl-item.done .tl-icon { color: var(--editor-accent, #16a34a); }
.tl-body { min-width: 0; }
.tl-title { font-weight: 500; color: var(--editor-primary, #333); word-break: break-all; }
.tl-detail { color: var(--editor-muted, #999); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 300px; }
.diff-card { border: 1px solid var(--editor-border, #e0e0e6); border-radius: 8px; overflow: hidden; }
.diff-head { display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--editor-inset-bg, #f4f4f2); font-size: 12px; }
.diff-path { font-weight: 500; color: var(--editor-primary, #333); word-break: break-all; }
.diff-count { color: var(--editor-accent, #16a34a); flex: none; }
.diff-body { margin: 0; padding: 8px 10px; font-size: 11px; line-height: 1.5; max-height: 120px; overflow: auto; color: var(--editor-muted, #666); white-space: pre-wrap; word-break: break-all; }
.ai-apply-row { display: flex; gap: 8px; }
</style>
