<template>
  <aside class="ag-panel">
    <div class="ag-head">
      <AiIcon :size="16" class="ag-logo" />
      <span class="ag-title">转换助手</span>
      <span v-if="running" class="ag-live">运行中</span>
      <button class="ag-close" title="收起" @click="$emit('close')"><CloseOutlined /></button>
    </div>

    <div class="ag-todos" v-if="todos.length">
      <div v-for="(t, i) in todos" :key="i" class="ag-todo" :class="t.status">
        <span class="ag-todo-dot"></span>{{ t.title }}
      </div>
    </div>

    <div class="ag-body" ref="bodyEl">
      <div v-if="timeline.length === 0" class="ag-welcome">
        <div class="ag-welcome-title">转换助手</div>
        <div class="ag-welcome-sub">可以问我项目结构、让它调整方向:如「把 service 层都剔除」「这个变量名改成 db_host」「用 bash 统计 TODO」。</div>
        <div class="ag-welcome-sub dim">过程与你的操作在同一条时间线里,随时打断、随时反悔(树上/变量面板直接改)。</div>
      </div>
      <template v-for="(e, i) in timeline" :key="i">
        <div v-if="e.kind === 'event'" class="ag-event">{{ e.text }}</div>
        <div v-else-if="e.kind === 'user'" class="ag-msg user">{{ e.text }}</div>
        <div v-else-if="e.kind === 'assistant'" class="ag-msg assistant">{{ e.text }}</div>
        <div v-else-if="e.kind === 'tool'" class="ag-tool" :class="{ open: expanded === i }">
          <div class="ag-tool-row" @click="expanded = expanded === i ? null : i">
            <span class="ag-tool-dot" :class="e.level"></span>
            <span class="ag-tool-name">{{ e.title }}</span>
            <span class="ag-tool-brief">{{ e.brief }}</span>
            <span class="ag-tool-chev">›</span>
          </div>
          <div v-if="expanded === i" class="ag-tool-detail">
            <pre class="ag-pre">{{ e.args }}</pre>
            <pre class="ag-pre out">{{ e.result }}</pre>
          </div>
        </div>
      </template>
    </div>

    <div class="ag-composer">
      <div class="ag-card">
        <textarea
          v-model="input"
          class="ag-input"
          rows="3"
          placeholder="发消息指挥助手,Enter 发送 / Shift+Enter 换行"
          :disabled="running"
          @keydown.enter.exact.prevent="submit()"
        ></textarea>
        <div class="ag-bar">
        <button class="ag-chip" @click="modelOpen = !modelOpen">{{ modelLabel }}<DownOutlined /></button>
        <button class="ag-chip" @click="thinkOpen = !thinkOpen">思考:{{ thinkLabel }}<DownOutlined /></button>
        <button class="ag-chip" @click="permOpen = !permOpen">{{ permLabel }}<DownOutlined /></button>
        <div class="ag-send">
          <button v-if="running" class="ag-stop" title="停止" @click="abortFlag = true"><PauseCircleOutlined /></button>
          <button v-else class="ag-go" :disabled="!input.trim()" title="发送 (Enter)" @click="submit()"><SendOutlined /></button>
        </div>
        </div>
      </div>
      <div v-if="modelOpen" class="ag-drop">
        <div class="ag-drop-item" @click="pickModel('', '')">默认模型</div>
        <template v-for="g in modelGroups" :key="g.provider">
          <div class="ag-drop-label">{{ g.provider }}</div>
          <div v-for="m in g.models" :key="m" class="ag-drop-item" :class="{ cur: selModel === m }" @click="pickModel(g.provider, m)">{{ m }}</div>
        </template>
      </div>
      <div v-if="thinkOpen" class="ag-drop">
        <div v-for="t in THINKS" :key="t.v" class="ag-drop-item" :class="{ cur: thinkLevel === t.v }" @click="thinkLevel = t.v; thinkOpen = false">{{ t.label }}<span class="ag-drop-desc">{{ t.desc }}</span></div>
      </div>
      <div v-if="permOpen" class="ag-drop wide">
        <div v-for="p in PERMS" :key="p.v" class="ag-drop-item" :class="{ cur: permMode === p.v }" @click="permMode = p.v; permOpen = false">
          <component :is="p.icon" class="ag-drop-ico" />
          <span>{{ p.label }}</span>
          <span class="ag-drop-desc">{{ p.desc }}</span>
        </div>
      </div>
    </div>

    <!-- bash/写操作确认(完全访问模式除外) -->
    <a-modal :open="confirming" title="工具执行确认" ok-text="执行" cancel-text="拒绝" @ok="resolveConfirm(true)" @cancel="resolveConfirm(false)">
      <div class="ag-confirm">
        <div class="ag-confirm-name">{{ confirmCall?.name }}</div>
        <pre class="ag-pre">{{ confirmCall?.argsText }}</pre>
      </div>
    </a-modal>
  </aside>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { CloseOutlined, DownOutlined, EditOutlined, ThunderboltOutlined, AuditOutlined, SafetyOutlined, SendOutlined, PauseCircleOutlined } from '@ant-design/icons-vue'
import AiIcon from '@/components/icons/AiIcon.vue'
import { useAIConfigStore } from '@/stores/ai-config'

const props = defineProps({
  snapshot: { type: Object, required: true }, // 工作台状态快照(每轮刷新进系统提示词)
  execOp: { type: Function, default: null },  // IR 操作同步执行器(name, args) => 结果文本
})
const emit = defineEmits(['close', 'op', 'log'])

// ---- composer 设置(与编辑器面板同源语义) ----
const THINKS = [
  { v: 'auto', label: '自动', desc: '跟随模型默认' },
  { v: 'off', label: '关', desc: '直接给结论' },
  { v: 'medium', label: '中', desc: '先简要分析再动手' },
  { v: 'high', label: '深', desc: '完整推演后执行' },
]
const PERMS = [
  { v: 'confirm', label: '变更前确认', desc: '写文件与 bash 逐条确认', icon: AuditOutlined, rounds: 12 },
  { v: 'autoEdit', label: '自动编辑', desc: '文件直写,bash 仍确认', icon: EditOutlined, rounds: 12 },
  { v: 'auto', label: '自动模式', desc: '同自动编辑,轮次 20', icon: ThunderboltOutlined, rounds: 20 },
  { v: 'full', label: '完全访问', desc: '全部自动,轮次 30', icon: SafetyOutlined, rounds: 30 },
]
const thinkLevel = ref(localStorage.getItem('ai-think-level') || 'auto')
const permMode = ref(localStorage.getItem('ai-perm-mode') || 'confirm')
watch(thinkLevel, (v) => localStorage.setItem('ai-think-level', v))
watch(permMode, (v) => localStorage.setItem('ai-perm-mode', v))
const maxRounds = computed(() => PERMS.find((p) => p.v === permMode.value)?.rounds || 12)
const thinkLabel = computed(() => THINKS.find((t) => t.v === thinkLevel.value)?.label || '自动')
const permLabel = computed(() => PERMS.find((p) => p.v === permMode.value)?.label || '')
const thinkDirective = () => ({
  off: '\n\n[思考级别:关] 直接给出结论,不展开长推理。',
  medium: '\n\n[思考级别:中] 先简要分析,再执行工具。',
  high: '\n\n[思考级别:深] 先推演方案(影响面/边界),再执行工具。',
}[thinkLevel.value] || '')

// 模型选择(aiConfig 分组)
const aiStore = useAIConfigStore()
const selProvider = ref(localStorage.getItem('ai-sel-provider') || '')
const selModel = ref(localStorage.getItem('ai-sel-model') || '')
watch(selProvider, (v) => localStorage.setItem('ai-sel-provider', v))
watch(selModel, (v) => localStorage.setItem('ai-sel-model', v))
const modelLabel = computed(() => selModel.value || '默认模型')
const modelOpen = ref(false)
const thinkOpen = ref(false)
const permOpen = ref(false)
const modelGroups = ref([])
const loadModels = async () => {
  try {
    await aiStore.loadAllProviders()
    const groups = []
    for (const p of (aiStore.providers || []).filter((p) => p.isEnabled)) {
      try { const ms = await aiStore.getProviderModelsGrouped(p.providerName); groups.push({ provider: p.providerName, models: (ms || []).flat ? (ms || []).flat() : [] }) } catch {}
    }
    modelGroups.value = groups
  } catch { /* 无配置时仅默认模型 */ }
}
const pickModel = (pn, m) => { selProvider.value = pn; selModel.value = m; modelOpen.value = false }
onMounted(loadModels)

// ---- 时间线 / 循环 ----
const timeline = ref([])
const taskMessages = ref([])
const todos = ref([])
const input = ref('')
const running = ref(false)
const abortFlag = ref(false)
const expanded = ref(null)
const bodyEl = ref(null)
const pushEvent = (text) => timeline.value.push({ kind: 'event', text })
const scrollBottom = async () => { await nextTick(); if (bodyEl.value) bodyEl.value.scrollTop = bodyEl.value.scrollHeight }
watch(() => timeline.value.length, scrollBottom)

// ---- FNV-1a 64(hex) —— 与 Rust fnv1a 一致,read→write 哈希新鲜度守卫 ----
const fnv1a64 = (s) => {
  let h = 0xcbf29ce484222325n
  for (let i = 0; i < s.length; i++) {
    h ^= BigInt(s.charCodeAt(i))
    h = (h * 0x100000001b3n) & 0xffffffffffffffffn
  }
  return h.toString(16).padStart(16, '0')
}

// ---- 工具 schema(转换宿主) ----
const str = (d) => ({ type: 'string', description: d })
const TOOLS = [
  { name: 'list_files', description: '列出镜像全部文件(含被剔除文件;跳过 .git)', parameters: { type: 'object', properties: {}, required: [] } },
  { name: 'read_file', description: '读取文件当前内容(编辑前必须先读;返回含哈希)', parameters: { type: 'object', properties: { path: str('文件相对路径') }, required: ['path'] } },
  { name: 'edit_file', description: '精确匹配替换:old_string 须与内容完全一致且唯一', parameters: { type: 'object', properties: { path: str('文件相对路径'), old_string: str('原文本(含缩进,须唯一)'), new_string: str('替换后文本') }, required: ['path', 'old_string', 'new_string'] } },
  { name: 'create_file', description: '创建新文件(全量内容;已存在会失败)', parameters: { type: 'object', properties: { path: str('新文件相对路径'), content: str('文件全量内容') }, required: ['path', 'content'] } },
  { name: 'bash', description: '在镜像目录执行 bash(git push 已被拦截;输出截断 64KB;默认 30s 超时)', parameters: { type: 'object', properties: { command: str('bash 命令'), timeout_ms: { type: 'integer', description: '超时毫秒(默认 30000,上限 120000)' } }, required: ['command'] } },
  { name: 'set_file_action', description: '调整工作台文件保留/剔除(立即生效)', parameters: { type: 'object', properties: { path: str('文件相对路径'), action: { type: 'string', enum: ['keep', 'exclude'], description: 'keep=保留进模板' } }, required: ['path', 'action'] } },
  { name: 'set_focus', description: '设置数据驱动的重点文件与 AI 暴露策略', parameters: { type: 'object', properties: { files: { type: 'array', items: { type: 'string' }, description: '重点文件路径列表' }, expose_all: { type: 'boolean', description: 'true=全部内容暴露,false=仅重点内容+目录结构' } }, required: ['files'] } },
  { name: 'update_variable', description: '修改变量(名称/默认值/启停)', parameters: { type: 'object', properties: { name: str('变量名'), new_name: str('新名称(可选)'), default_value: str('新默认值(可选)'), enabled: { type: 'boolean', description: '启用/停用(可选)' } }, required: ['name'] } },
  { name: 'rerun_analysis', description: '触发重新分析(异步;结果看转换过程流)', parameters: { type: 'object', properties: {}, required: [] } },
  { name: 'update_todo', description: '维护任务计划(整体替换)', parameters: { type: 'object', properties: { items: { type: 'array', items: { type: 'object', properties: { title: { type: 'string' }, status: { type: 'string', description: 'pending|in_progress|done' } } } } }, required: ['items'] } },
]

// ---- 确认门:bash 恒确认(完全访问除外);confirm 模式下写文件也确认 ----
const confirming = ref(false)
const confirmCall = ref(null)
let confirmResolver = null
const askConfirm = (call) => new Promise((resolve) => {
  confirmCall.value = { name: call.name, argsText: JSON.stringify(call.arguments, null, 2) }
  confirming.value = true
  confirmResolver = resolve
})
const resolveConfirm = (ok) => { confirming.value = false; confirmResolver?.(ok); confirmResolver = null }
const needsConfirm = (name) => {
  if (permMode.value === 'full') return false
  if (name === 'bash') return true
  if (permMode.value === 'confirm' && ['edit_file', 'create_file'].includes(name)) return true
  return false
}

// ---- 工具执行 ----
async function execTool(name, args) {
  switch (name) {
    case 'list_files': {
      const raw = await invoke('convert_agent_list', { root: props.snapshot.dir, sub: args.sub || null })
      const v = JSON.parse(raw)
      const lines = v.files.map((f) => f.path)
      return (v.truncated ? '(超 2000 条已截断)\n' : '') + (lines.join('\n') || '(空)')
    }
    case 'read_file': {
      const raw = await invoke('convert_read_file', { root: props.snapshot.dir, path: args.path })
      const v = JSON.parse(raw)
      const c = v.content || ''
      const head = `哈希: ${fnv1a64(c)}(edit 时作为 expect_hash 传入)\n`
      return head + (c.length > 24000 ? c.slice(0, 24000) + '\n...(截断)' : (c || '(空文件)'))
    }
    case 'edit_file': {
      const raw = await invoke('convert_read_file', { root: props.snapshot.dir, path: args.path })
      const cur = JSON.parse(raw).content || ''
      const hash = fnv1a64(cur)
      const first = cur.indexOf(args.old_string)
      if (first === -1) return '错误:old_string 未找到(注意缩进与完全一致),请重读。'
      if (cur.indexOf(args.old_string, first + 1) !== -1) return '错误:old_string 出现多次,请扩大范围。'
      const next = cur.slice(0, first) + args.new_string + cur.slice(first + args.old_string.length)
      await invoke('convert_agent_write', { root: props.snapshot.dir, path: args.path, content: next, expectHash: hash })
      emit('log', `edit_file ${args.path}(${args.new_string.length - args.old_string.length >= 0 ? '+' : ''}${args.new_string.length - args.old_string.length} 字符)`)
      return '已写入。'
    }
    case 'create_file': {
      await invoke('convert_agent_write', { root: props.snapshot.dir, path: args.path, content: args.content, expectHash: null })
      emit('log', `create_file ${args.path}`)
      return '已创建。'
    }
    case 'bash': {
      const raw = await invoke('convert_agent_bash', { root: props.snapshot.dir, command: args.command, timeoutMs: args.timeout_ms || null })
      const v = JSON.parse(raw)
      emit('log', `bash: ${args.command.slice(0, 120)} → exit ${v.exitCode}`)
      return `exit ${v.exitCode}\n${v.output || '(无输出)'}`
    }
    case 'set_file_action':
    case 'set_focus':
    case 'update_variable':
    case 'rerun_analysis': {
      const r = props.execOp ? props.execOp(name, args) : null
      return r || '已提交工作台执行。'
    }
    case 'update_todo': {
      todos.value = (args.items || []).filter((x) => x && x.title).map((x) => ({ title: String(x.title), status: x.status || 'pending' }))
      return `计划已更新(${todos.value.length} 项)`
    }
    default:
      return `错误:未知工具 ${name}`
  }
}
const safeExec = async (call) => {
  try { return String(await execTool(call.name, call.arguments || {})) }
  catch (e) { return `错误: ${e}` }
}

// ---- 系统提示词(每轮刷新快照) ----
const buildSystemPrompt = () => {
  const s = props.snapshot
  const vars = (s.variables || []).slice(0, 30).map((v) => `${v.name}=${v.defaultValue}`).join(', ')
  const files = (s.files || [])
  const kept = files.filter((f) => f.action === 'keep')
  const focus = s.focusFiles?.length ? `重点文件(${s.focusFiles.length},exposeAll=${s.exposeAll}): ${s.focusFiles.slice(0, 15).join(', ')}` : '未设置重点文件'
  const logTail = (s.logTail || []).slice(-8).map((l) => `[${l.stage}] ${l.text}`).join('\n')
  return [
    '你是「模板转换」工作台的助手。工作台正在把一个 git 项目转换为模板(脚手架或数据驱动),用户会随时指挥你调整方向。',
    '工具约定:',
    '- 文件工具作用于本地镜像目录(可抛弃副本,git 可恢复);edit 前必须 read_file(哈希守卫会拒绝过期写入)。',
    '- bash 在镜像目录内执行,push 类命令已被拦截;优先用 rg/grep 等只读命令了解项目。',
    '- IR 操作(set_file_action/set_focus/update_variable/rerun_analysis)直接改变工作台状态,用户在界面上立即可见、可反悔。',
    '- 修改文件或调整重点后,建议提示用户(或直接调用)rerun_analysis 让结果生效。',
    '回答用中文,简洁;行动优先,解释其次。',
    '',
    `## 当前工作台状态`,
    `来源: ${s.source || '?'}(${s.packId || '?'},类型 ${s.tplType})`,
    `文件: 保留 ${kept.length} / 剔除 ${files.length - kept.length}`,
    focus,
    `变量(${s.variables?.length || 0}): ${vars || '(无)'}`,
    s.logTail?.length ? `近期过程:\n${logTail}` : '',
  ].filter(Boolean).join('\n')
}

// ---- 上下文修剪(与编辑器同策略的简化版) ----
const estTokens = (arr) => Math.ceil(JSON.stringify(arr).length / 3)
const trimContext = () => {
  if (estTokens(taskMessages.value) <= 40000 * 0.85) return
  const head = taskMessages.value.length && taskMessages.value[0].role === 'system' ? [taskMessages.value[0]] : []
  const rest = taskMessages.value.slice(head.length)
  const keep = 8
  const mid = rest.slice(0, Math.max(0, rest.length - keep)).map((m) =>
    m.role === 'tool_result' && String(m.content || '').length > 80 ? { ...m, content: '(已折叠的历史工具结果)' } : m
  )
  taskMessages.value = [...head, ...mid, ...rest.slice(-keep)]
}

const submit = (textArg) => {
  const text = (textArg ?? input.value).trim()
  if (!text || running.value) return
  input.value = ''
  runAgent(text)
}

const runAgent = async (text) => {
  running.value = true
  abortFlag.value = false
  timeline.value.push({ kind: 'user', text })
  try {
    if (!taskMessages.value.length || taskMessages.value[0].role !== 'system') {
      taskMessages.value.unshift({ role: 'system', content: buildSystemPrompt() })
    } else {
      taskMessages.value[0].content = buildSystemPrompt() // 快照刷新
    }
    taskMessages.value.push({ role: 'user', content: text + thinkDirective() })
    for (let round = 0; round < maxRounds.value; round++) {
      if (abortFlag.value) { pushEvent('已中止'); break }
      trimContext()
      const raw = await invoke('ai_agent_turn', {
        messages: taskMessages.value,
        tools: TOOLS,
        provider: selProvider.value || null,
        model: selModel.value || null,
        thinking: thinkLevel.value === 'auto' ? null : thinkLevel.value,
      })
      const r = JSON.parse(raw)
      if (r.type === 'final') {
        if (r.text) { timeline.value.push({ kind: 'assistant', text: r.text }) }
        taskMessages.value.push({ role: 'assistant', content: r.text || '' })
        break
      }
      taskMessages.value.push({ role: 'assistant', content: r.text || '', tool_calls: r.calls })
      if (r.text) timeline.value.push({ kind: 'assistant', text: r.text })
      for (const c of r.calls) {
        if (abortFlag.value) break
        let out
        if (needsConfirm(c.name)) {
          const ok = await askConfirm(c)
          out = ok ? await safeExec(c) : '用户拒绝执行该工具。'
        } else {
          out = await safeExec(c)
        }
        timeline.value.push({
          kind: 'tool', level: c.name === 'bash' ? 'bash' : (['edit_file', 'create_file'].includes(c.name) ? 'write' : 'read'),
          title: c.name,
          brief: briefOf(c),
          args: JSON.stringify(c.arguments, null, 2),
          result: String(out).slice(0, 4000),
        })
        taskMessages.value.push({ role: 'tool_result', tool_call_id: c.id, name: c.name, content: String(out).slice(0, 8000) })
      }
    }
  } catch (e) {
    timeline.value.push({ kind: 'assistant', text: '调用失败: ' + (e.message || e) })
  } finally {
    running.value = false
  }
}

const briefOf = (c) => {
  const a = c.arguments || {}
  if (c.name === 'bash') return String(a.command || '').slice(0, 60)
  if (a.path) return a.path
  if (c.name === 'rerun_analysis') return ''
  return ''
}

defineExpose({ submit })
</script>

<style scoped>
.ag-panel { width: 380px; flex-shrink: 0; display: flex; flex-direction: column; background: var(--ai-dock-bg, var(--editor-panel-bg, #fff)); border-left: 1px solid var(--editor-border, #e2e8f0); min-height: 0; position: relative; }
.ag-head { height: 40px; flex-shrink: 0; display: flex; align-items: center; gap: 8px; padding: 0 10px; border-bottom: 1px solid var(--editor-border, #e2e8f0); }
.ag-logo { color: var(--color-brand, #16a34a); font-size: 15px; }
.ag-title { font-size: 12.5px; font-weight: 600; color: var(--color-text, #1b1c1f); flex: 1; }
.ag-live { font-size: 10px; color: #d97706; animation: ag-blink 1.2s ease-in-out infinite; }
@keyframes ag-blink { 0%, 100% { opacity: 0.4; } 50% { opacity: 1; } }
.ag-close { border: none; background: transparent; color: var(--color-text-secondary, #64748b); cursor: pointer; font-size: 12px; padding: 4px; border-radius: 6px; }
.ag-close:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }

.ag-todos { flex-shrink: 0; max-height: 110px; overflow-y: auto; padding: 6px 12px; border-bottom: 1px solid var(--editor-border, #e2e8f0); display: flex; flex-direction: column; gap: 3px; }
.ag-todo { display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--color-text-secondary, #64748b); }
.ag-todo-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-border, #ccc); flex: none; }
.ag-todo.done .ag-todo-dot { background: var(--color-brand, #16a34a); }
.ag-todo.done { text-decoration: line-through; opacity: 0.6; }
.ag-todo.in_progress .ag-todo-dot { background: #d97706; }

.ag-body { flex: 1; min-height: 0; overflow-y: auto; padding: 10px 12px; display: flex; flex-direction: column; gap: 8px; }
.ag-welcome { padding: 24px 8px; text-align: center; }
.ag-welcome-title { font-size: 15px; font-weight: 600; color: var(--color-text, #1b1c1f); margin-bottom: 8px; }
.ag-welcome-sub { font-size: 12px; color: var(--color-text-secondary, #64748b); line-height: 1.7; }
.ag-welcome-sub.dim { color: var(--color-text-muted, #9aa0a6); margin-top: 6px; }

.ag-event { text-align: center; font-size: 11px; color: var(--color-text-muted, #9aa0a6); padding: 2px 0; }
.ag-msg { font-size: 12.5px; line-height: 1.7; white-space: pre-wrap; word-break: break-word; border-radius: 8px; padding: 8px 10px; }
.ag-msg.user { align-self: flex-end; max-width: 92%; background: var(--color-nav-active, #eef0ec); color: var(--color-text, #1b1c1f); }
.ag-msg.assistant { align-self: flex-start; max-width: 100%; color: var(--color-text, #333); }

.ag-tool { border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; overflow: hidden; }
.ag-tool-row { display: flex; align-items: center; gap: 8px; padding: 5px 10px; cursor: pointer; font-size: 11.5px; }
.ag-tool-row:hover { background: var(--color-surface, #f7f7f5); }
.ag-tool-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; background: var(--color-text-muted, #9aa0a6); }
.ag-tool-dot.write { background: #3e7bfa; }
.ag-tool-dot.bash { background: #8b5cf6; }
.ag-tool-name { font-weight: 600; font-family: Consolas, monospace; color: var(--color-text, #333); }
.ag-tool-brief { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--color-text-muted, #9aa0a6); font-family: Consolas, monospace; }
.ag-tool-chev { color: var(--color-text-muted, #9aa0a6); transition: transform 0.15s; }
.ag-tool.open .ag-tool-chev { transform: rotate(90deg); }
.ag-tool-detail { border-top: 1px solid var(--editor-border, #e2e8f0); padding: 8px 10px; display: flex; flex-direction: column; gap: 6px; }
.ag-pre { margin: 0; padding: 8px; background: var(--color-canvas, #f6f8fa); border-radius: 6px; font-family: Consolas, 'JetBrains Mono', monospace; font-size: 11px; line-height: 1.55; white-space: pre-wrap; word-break: break-all; max-height: 220px; overflow-y: auto; color: var(--color-text, #333); }
.ag-pre.out { color: var(--color-text-secondary, #555); }

.ag-composer { flex-shrink: 0; padding: 10px 12px; position: relative; }
/* 输入卡片:textarea 与 chips 同卡(chips 在卡内底边,ZCode 式) */
.ag-card { border: 1px solid var(--editor-border, #e0e0e6); border-radius: 10px; background: var(--ai-composer-bg, var(--editor-panel-bg, #fff)); overflow: hidden; transition: border-color 0.15s ease; }
.ag-card:focus-within { border-color: var(--color-brand, #16a34a); }
.ag-input { width: 100%; border: none; outline: none; resize: none; padding: 8px 10px 4px; font-size: 12.5px; font-family: inherit; background: transparent; color: var(--color-text, #1b1c1f); }
.ag-input:disabled { opacity: 0.6; }
.ag-bar { display: flex; align-items: center; gap: 4px; padding: 2px 6px 6px 8px; }
.ag-chip { display: inline-flex; align-items: center; gap: 4px; border: none; background: transparent; font-size: 11px; color: var(--color-text-secondary, #64748b); padding: 3px 8px; border-radius: 6px; cursor: pointer; }
.ag-chip:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.ag-send { margin-left: auto; display: flex; gap: 6px; }
.ag-go { width: 26px; height: 26px; border: none; border-radius: 6px; background: var(--color-primary, #1b1c1f); color: var(--color-text-light, #fff); display: inline-flex; align-items: center; justify-content: center; font-size: 13px; cursor: pointer; }
.ag-go:disabled { background: var(--color-surface, #ececea); color: var(--color-text-muted, #a6a8ad); cursor: not-allowed; }
.ag-stop { width: 26px; height: 26px; border: none; border-radius: 6px; background: rgba(220, 38, 38, 0.08); color: #dc2626; display: inline-flex; align-items: center; justify-content: center; font-size: 13px; cursor: pointer; }

.ag-drop { position: absolute; bottom: 100%; left: 12px; right: 12px; max-height: 260px; overflow-y: auto; background: var(--editor-panel-bg, #fff); border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; box-shadow: 0 8px 24px rgba(0,0,0,0.12); padding: 4px; z-index: 20; margin-bottom: 4px; }
.ag-drop.wide { left: auto; width: 320px; }
.ag-drop-label { font-size: 10.5px; font-weight: 600; color: var(--color-text-muted, #9aa0a6); padding: 6px 10px 2px; }
.ag-drop-item { display: flex; align-items: center; gap: 8px; padding: 6px 10px; font-size: 12px; color: var(--color-text, #333); border-radius: 6px; cursor: pointer; }
.ag-drop-item:hover { background: var(--color-hover, #f1f5f9); }
.ag-drop-item.cur { color: var(--color-brand, #16a34a); font-weight: 600; }
.ag-drop-desc { font-size: 10.5px; color: var(--color-text-muted, #9aa0a6); margin-left: auto; }
.ag-drop-ico { font-size: 13px; color: var(--color-text-secondary, #64748b); }

.ag-confirm-name { font-weight: 600; font-family: Consolas, monospace; font-size: 13px; margin-bottom: 8px; }
</style>
