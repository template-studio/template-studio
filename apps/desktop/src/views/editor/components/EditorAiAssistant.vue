<template>
  <!-- 停靠式右侧 AI 栏:单一对话窗口(问答/编辑任务同管线,问答不触发写工具) -->
  <div v-show="open" class="ai-dock" :style="{ width: width + 'px' }">
    <div class="ai-resize-handle" @mousedown="startResize"></div>

    <div class="ai-dock-head">
      <div class="ai-dock-title">
        <AiIcon :size="16" class="ai-title-icon" />
        <span>AI 助手</span>
      </div>
      <div class="ai-dock-head-right">
        <span v-if="totalTokens > 0" class="ai-token-meter" :title="`累计 token(入 ${tokIn}/出 ${tokOut})`">
          {{ (totalTokens / 1000).toFixed(1) }}k tok
        </span>
        <a-button v-if="!busy" type="text" size="small" title="会话历史" @click="toggleHistory">
          <template #icon><HistoryOutlined /></template>
        </a-button>
        <a-button v-if="!busy && timeline.length > 0" type="text" size="small" @click="resetAgent">重置</a-button>
        <a-button type="text" size="small" @click="open = false">
          <template #icon><CloseOutlined /></template>
        </a-button>
      </div>
    </div>

    <div class="ai-body">
      <!-- 会话历史列表 -->
      <div v-if="historyOpen" class="history-box">
        <div class="history-head">
          <span>会话历史</span>
          <span class="history-count">{{ historyList.length }}</span>
        </div>
        <div v-if="historyLoading" class="history-empty">加载中…</div>
        <div v-else-if="historyList.length === 0" class="history-empty">暂无历史会话</div>
        <template v-else>
          <div
            v-for="s in historyList" :key="s.name"
            class="history-item"
            :class="{ cur: s.name === (sessionName ? sessionName + '.jsonl' : '') }"
            @click="switchSession(s.name)"
            >
              <span class="history-time">{{ fmtTime(s.mtimeMs) }}</span>
              <span class="history-size">{{ fmtSize(s.size) }}</span>
              <span class="history-del" title="删除会话" @click.stop="removeSession(s.name)">
                <DeleteOutlined />
              </span>
            </div>
          </template>
        </div>
        <Welcome v-if="timeline.length === 0" class="ai-welcome"
          :icon="() => h('span', { class: 'ai-welcome-icon' }, [h(AiIcon, { size: 36 })])"
          title="AI 助手" description="提问或下达编辑任务,支持 @ 引用文件" />
        <div v-else class="agent-stream">
          <!-- 工作计时(ZCode 式) -->
          <div v-if="agentRunning && elapsedText" class="working-row">{{ elapsedText }}</div>
          <!-- 紧凑步骤行(终端式:单行折叠,点击展开);answer 为 AI 文本回复 -->
          <div class="steps">
            <template v-for="(e, i) in timeline" :key="i">
              <div v-if="e.kind === 'answer'" class="answer">{{ e.text }}</div>
              <div v-else class="step" :class="[e.kind, { open: expandedStep === i }]" @click="toggleStep(i)">
                <span class="step-dot"></span>
                <span class="step-title">{{ e.title }}</span>
                <span v-if="e.detail" class="step-detail">{{ e.detail }}</span>
                <span v-if="e.full" class="step-chev">›</span>
              </div>
            </template>
            <div v-if="expandedStep !== null && timeline[expandedStep]?.full" class="step-full">{{ timeline[expandedStep].full }}</div>
          </div>

          <!-- 计划清单 -->
          <div v-if="todos.length > 0" class="todo-box">
            <div v-for="(td, i) in todos" :key="i" class="todo-item" :class="td.status">
              <span class="todo-dot">{{ td.status === 'done' ? '●' : td.status === 'in_progress' ? '◐' : '○' }}</span>
              <span class="todo-title">{{ td.title }}</span>
            </div>
          </div>

          <!-- diff 卡片:汇总头 + 逐文件 -->
          <div v-if="dirtyFiles.length > 0" class="diffs-head">
            {{ dirtyFiles.length }} 个文件已更改
            <span class="applied-plus">+{{ dirtyAdded }}</span>
            <span class="applied-minus">-{{ dirtyRemoved }}</span>
          </div>
          <div v-for="(d, i) in dirtyFiles" :key="'d' + i" class="diff-card">
            <div class="diff-head">
              <span class="diff-path">{{ d.path }}</span>
              <span class="diff-count">+{{ d.added }} / -{{ d.removed }}</span>
            </div>
            <pre class="diff-body">{{ d.preview }}</pre>
          </div>

          <!-- 应用即 checkpoint:快照锚点与一键撤销 -->
          <div v-if="appliedInfo && appliedInfo.count > 0" class="applied-bar">
            <span class="applied-text">
              已应用 {{ appliedInfo.count }} 项修改
              <span class="applied-plus">+{{ appliedInfo.added || 0 }}</span>
              <span class="applied-minus">-{{ appliedInfo.removed || 0 }}</span>
              · 修改前快照 <b>{{ appliedInfo.version }}</b>
            </span>
            <a-popconfirm
              title="回滚将丢弃该快照之后的全部更改（含手动修改），确定？"
              ok-text="回滚" cancel-text="取消" @confirm="undoApply"
            >
              <a-button size="small" danger :loading="undoing">撤销</a-button>
            </a-popconfirm>
          </div>
        </div>
      </div>

      <div class="ai-sender-wrap">
        <div v-if="dirtyFiles.length > 0 && !busy" class="ai-apply-row">
          <a-button type="primary" size="small" :loading="applying" @click="applyAll">应用全部修改({{ dirtyFiles.length }})</a-button>
          <a-button size="small" :disabled="applying" @click="discardAll">全部放弃</a-button>
        </div>
        <div v-if="atMatches.length > 0" class="at-panel">
          <button v-for="p in atMatches" :key="p" class="at-item" @mousedown.prevent="pickAt(p)">{{ p }}</button>
        </div>
        <!-- 输入框卡片:Sender + 底部 chips 同框(ZCode 式) -->
        <div class="ai-composer">
          <Sender v-model:value="agentInput" :loading="agentRunning" :disabled="applying"
            placeholder="提问或描述编辑任务,支持 @ 引用文件"
            submit-type="enter" @submit="runAgent" @cancel="abortAgent" @focus="loadFilePaths" />

    <!-- composer 底栏:模型 / 思考级别 / 权限访问模式(向上弹出) -->
    <div class="ai-composer-bar">
      <a-popover v-model:open="modelPopover" trigger="click" placement="topLeft" :overlay-style="{ maxWidth: '300px' }">
        <template #content>
          <div class="mp-list">
            <div class="mp-group">默认</div>
            <div class="mp-item" :class="{ cur: !selProvider }" @click="pickDefaultModel">
              <span class="mp-name">跟随设置中的默认提供商</span>
            </div>
            <template v-for="p in enabledProviders" :key="p.providerName">
              <div class="mp-group">{{ p.displayName || p.providerName }}</div>
              <template v-for="g in modelGroupsCache[p.providerName] || []" :key="g.groupId">
                <div v-if="g.groupName && (modelGroupsCache[p.providerName] || []).length > 1" class="mp-subgroup">{{ g.groupName }}</div>
                <div v-for="m in g.models" :key="m.id" class="mp-item"
                  :class="{ cur: selProvider === p.providerName && selModel === m.modelId }"
                  @click="pickModel(p.providerName, m.modelId)">
                  <span class="mp-name">{{ m.modelName || m.modelId }}</span>
                  <span v-if="m.supportsFunctions" class="mp-fn" title="支持工具调用">fn</span>
                </div>
              </template>
              <div v-if="!(modelGroupsCache[p.providerName] || []).length" class="mp-empty">暂无模型</div>
            </template>
          </div>
        </template>
        <button class="chip chip-model" title="选择模型">
          {{ selModel || '默认模型' }}<span class="chip-caret">▾</span>
        </button>
      </a-popover>

      <a-popover trigger="click" placement="topLeft">
        <template #content>
          <div class="mp-list">
            <div v-for="t in THINKS" :key="t.v" class="mp-item" :class="{ cur: thinkLevel === t.v }" @click="thinkLevel = t.v">
              <span class="mp-name">{{ t.label }}</span>
              <span class="mp-desc">{{ t.desc }}</span>
            </div>
          </div>
        </template>
        <button class="chip" title="思考级别"><BulbOutlined class="chip-ico" />思考·{{ THINKS.find((t) => t.v === thinkLevel)?.label }}<span class="chip-caret">▾</span></button>
      </a-popover>

      <a-popover trigger="click" placement="topLeft">
        <template #content>
          <div class="perm-list">
            <div v-for="pm in PERMS" :key="pm.v" class="perm-item" :class="[pm.v, { cur: permMode === pm.v }]" @click="permMode = pm.v">
              <component :is="pm.icon" class="perm-ico" />
              <div class="perm-text">
                <div class="perm-name">
                  {{ pm.label }}
                  <CheckOutlined v-if="permMode === pm.v" class="perm-check" />
                </div>
                <div class="perm-desc">{{ pm.desc }}</div>
              </div>
            </div>
          </div>
        </template>
        <button class="chip" :class="'chip-perm-' + permMode" title="权限访问模式">
          <SafetyOutlined class="chip-ico" />{{ PERMS.find((p) => p.v === permMode)?.label }}<span class="chip-caret">▾</span>
        </button>
      </a-popover>
    </div>
        </div>
      </div>
  </div>

  <!-- 收起态迷你徽标:运行中转圈点/有未应用修改绿点 -->
  <button
    v-if="!open && (busy || dirtyFiles.length > 0)"
    class="ai-mini"
    :title="busy ? 'AI 代理执行中' : '有未应用的修改'"
    @click="open = true"
  >
    <AiIcon :size="16" />
    <span v-if="busy" class="ai-mini-dot spin"></span>
    <span v-else class="ai-mini-dot"></span>
  </button>
</template>

<script setup>
import { ref, watch, nextTick, reactive, computed, onMounted, onUnmounted, h } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { CloseOutlined, HistoryOutlined, DeleteOutlined, SafetyOutlined, BulbOutlined, AuditOutlined, EditOutlined, ThunderboltOutlined, CheckOutlined } from '@ant-design/icons-vue'
import { Sender, Welcome } from 'ant-design-x-vue'
import { useAIConfigStore } from '@/stores/ai-config'
import AiIcon from '@/components/icons/AiIcon.vue'
import { getTemplateFileTree, getTemplateFileContent, editTemplateFile, addTemplateFile } from '@/api/editor/templateFiles'
import { createRelease, rollbackVersion } from '@/api/editor/releases'

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

// ===== 单一对话窗口(问答与编辑任务同走 agent 管线) =====
const buildExtraContext = () => {
  const parts = []
  if (props.currentFilePath) parts.push(`当前打开的文件: ${props.currentFilePath}`)
  const names = props.templateVariables.map((v) => v.fieldName || v.name).filter(Boolean).slice(0, 40)
  if (names.length > 0) parts.push(`模板已有变量: ${names.join(', ')}`)
  return parts.join('\n')
}

// ===== 编辑代理管线 =====
const agentInput = ref('')
const agentRunning = ref(false)
const applying = ref(false)
// 应用即 checkpoint:最近一次应用的修改前快照锚点({ count, version })
const appliedInfo = ref(null)
const undoing = ref(false)
// 工作计时(运行中每秒走字,ZCode 式「工作中 X 分 X 秒」)
const agentStartAt = ref(0)
const nowTick = ref(0)
let tickTimer = null
const elapsedText = computed(() => {
  if (!agentRunning.value || !agentStartAt.value) return ''
  const s = Math.max(0, Math.floor((nowTick.value - agentStartAt.value) / 1000))
  const m = Math.floor(s / 60)
  return m > 0 ? `工作中 ${m} 分 ${s % 60} 秒` : `工作中 ${s} 秒`
})

// ---- composer 底栏:模型/思考级别/权限访问模式(参考主流 coding agent) ----
const THINKS = [
  { v: 'auto', label: '自动', desc: '跟随模型默认' },
  { v: 'off', label: '关', desc: '直接给结论,不展开推理' },
  { v: 'medium', label: '中', desc: '先简要分析再动手' },
  { v: 'high', label: '深', desc: '完整推演方案后执行' },
]
const PERMS = [
  { v: 'confirm', label: '变更前确认', desc: '改动先审查,手动应用', icon: AuditOutlined, rounds: 12 },
  { v: 'autoEdit', label: '自动编辑', desc: '改动即时写入,可一键撤销', icon: EditOutlined, rounds: 12 },
  { v: 'auto', label: '自动模式', desc: '自动编辑,轮次上限 20', icon: ThunderboltOutlined, rounds: 20 },
  { v: 'full', label: '完全访问', desc: '自动编辑,轮次上限 30', icon: SafetyOutlined, rounds: 30 },
]
const thinkLevel = ref(localStorage.getItem('ai-think-level') || 'auto')
const permMode = ref(localStorage.getItem('ai-perm-mode') || 'confirm')
watch(thinkLevel, (v) => localStorage.setItem('ai-think-level', v))
watch(permMode, (v) => localStorage.setItem('ai-perm-mode', v))
const maxRounds = computed(() => PERMS.find((p) => p.v === permMode.value)?.rounds || 12)
// 思考级别的提示词指令(wire 级参数之外,对不支持思考参数的模型仍生效)
const thinkDirective = () => ({
  off: '\n\n[思考级别:关] 直接给出结论与修改,不展开长推理。',
  medium: '\n\n[思考级别:中] 先简要分析,再执行工具调用。',
  high: '\n\n[思考级别:深] 先在内部完整推演方案(多文件影响、边界情况),再执行工具调用。',
}[thinkLevel.value] || '')
const turnOptions = () => ({
  provider: selProvider.value || null,
  model: selModel.value || null,
  thinking: thinkLevel.value === 'auto' ? null : thinkLevel.value,
})

// 模型选择:按 provider 分组,弹出时惰性加载各启用提供商的模型分组
const aiStore = useAIConfigStore()
const selProvider = ref(localStorage.getItem('ai-sel-provider') || '')
const selModel = ref(localStorage.getItem('ai-sel-model') || '')
watch(selProvider, (v) => localStorage.setItem('ai-sel-provider', v))
watch(selModel, (v) => localStorage.setItem('ai-sel-model', v))
const modelPopover = ref(false)
const modelGroupsCache = reactive({})
const enabledProviders = computed(() => (aiStore.providers || []).filter((p) => p.isEnabled))
const ensureModels = async (pn) => {
  if (modelGroupsCache[pn]) return
  modelGroupsCache[pn] = []
  modelGroupsCache[pn] = await aiStore.getProviderModelsGrouped(pn)
}
watch(modelPopover, async (open) => {
  if (!open) return
  await aiStore.loadAllProviders()
  enabledProviders.value.forEach((p) => ensureModels(p.providerName))
})
onMounted(() => { aiStore.loadAllProviders().catch(() => {}) })
const modelLabel = () => selModel.value || '默认模型'
// 切换留痕:agent 模式下写入时间线(ZCode 式居中分隔事件)
const pushModelEvent = (from) => {
  const to = modelLabel()
  if (from === to) return
  timeline.value.push({ kind: 'tool', title: '模型已切换', detail: `${from} → ${to}` })
}
const pickModel = (pn, m) => {
  const from = modelLabel()
  selProvider.value = pn
  selModel.value = m
  modelPopover.value = false
  pushModelEvent(from)
}
const pickDefaultModel = () => {
  const from = modelLabel()
  selProvider.value = ''
  selModel.value = ''
  modelPopover.value = false
  pushModelEvent(from)
}
const timeline = ref([])
const abortFlag = ref(false)
const taskMessages = ref([])
const todos = ref([])
const filePaths = ref([])
const loadFilePaths = async () => {
  if (filePaths.value.length > 0) return
  try {
    const res = await getTemplateFileTree(tid())
    const paths = []
    const walk = (nodes) => nodes.forEach((n) => {
      if (n.isDirectory || n.is_directory) { walk(n.children || []) } else { paths.push(n.filePath || n.file_path) }
    })
    walk(res.data?.data?.tree || [])
    filePaths.value = paths
  } catch {}
}
const atSuffix = computed(() => {
  const m = /(^|\s)@([\w\/.\-]*)$/.exec(agentInput.value || '')
  return m ? m[2] : null
})
const atMatches = computed(() =>
  atSuffix.value === null ? [] : filePaths.value.filter((p) => p.includes(atSuffix.value)).slice(0, 8)
)
// ---- 上下文预算与修剪(压缩层:模型摘要优先,失败回落规则折叠) ----
const CTX_BUDGET = 40000  // 安全窗 token(粗估)
const estTokens = (arr) => Math.ceil(JSON.stringify(arr).length / 3)
const trimContext = () => {
  if (estTokens(taskMessages.value) <= CTX_BUDGET * 0.85) return false
  const arr = taskMessages.value
  const head = arr.length > 0 && arr[0].role === 'system' ? [arr[0]] : []
  const rest = arr.slice(head.length)
  const keep = 8  // 保留最近 8 条原文
  const mid = rest.slice(0, Math.max(0, rest.length - keep)).map((m) =>
    m.role === 'tool_result' && String(m.content || '').length > 80
      ? { ...m, content: '(已折叠的历史工具结果)' }
      : m
  )
  taskMessages.value = [...head, ...mid, ...rest.slice(-keep)]
  return true
}

// 模型生成结构化交接摘要,替换整个被折叠中段(摘要本身作为普通消息,后续压缩会再次吸收它)
const compactContext = async () => {
  const arr = taskMessages.value
  const head = arr.length > 0 && arr[0].role === 'system' ? [arr[0]] : []
  const rest = arr.slice(head.length)
  const keep = 8
  const folded = rest.slice(0, Math.max(0, rest.length - keep)).slice(-24) // 至多回看 24 条,更早已是占位
  if (folded.length === 0) { trimContext(); return null }
  try {
    const digest = folded.map((m) => ({
      role: m.role,
      content: String(m.content || '').slice(0, 1200),
      tool_calls: Array.isArray(m.tool_calls) ? m.tool_calls.map((c) => ({ name: c.name, arguments: c.arguments })) : undefined,
    }))
    const result = await invoke('ai_chat', {
      message: `下面是一个模板编辑代理的工作历史(按时间序,JSON)。请压缩为结构化交接摘要,供下一轮继续工作。严格按以下格式输出,每节至多 6 条、每条一行,没有则写"无":\n## 已完成\n## 进行中\n## 已修改文件(路径+一句话)\n## 技术决策及原因\n## 用户约束\n## 下一步\n\n---\n${JSON.stringify(digest)}`,
      templatePath: null, projectId: null, extraContext: null, history: [],
    })
    const text = (JSON.parse(result).response || '').trim()
    if (!text) throw new Error('空摘要')
    taskMessages.value = [
      ...head,
      { role: 'user', content: `[交接摘要·替代更早历史]\n${text}` },
      ...rest.slice(-keep),
    ]
    return { text }
  } catch {
    trimContext() // 回落:规则折叠工具结果
    return null
  }
}

const pickAt = (path) => {
  agentInput.value = (agentInput.value || '').replace(/@([\w\/.\-]*)$/, '@' + path + ' ')
}

// ===== 会话持久化(localStorage,按模板隔离;续跑时工具结果已裁剪) =====
const sessionName = ref(null) // 当前会话时间戳名;null=新会话
const saveTimer = { t: null }
const scheduleSave = () => {
  clearTimeout(saveTimer.t)
  saveTimer.t = setTimeout(saveSession, 500)
}
const saveSession = async () => {
  // 全空态不落盘:避免重置/删当前会话后被 watcher 复活成空会话文件
  if (
    taskMessages.value.length === 0 &&
    timeline.value.length === 0 && todos.value.length === 0 && workset.size === 0
  ) return
  try {
    const trimmed = taskMessages.value.map((m) =>
      m.role === 'tool_result'
        ? { ...m, content: String(m.content || '').slice(0, 2000) }
        : m
    )
    if (!sessionName.value) sessionName.value = String(Date.now())
    const line = (o) => JSON.stringify(o)
    const lines = [
      line({ t: 'meta', tokIn: tokIn.value, tokOut: tokOut.value, savedAt: Date.now() }),
      ...todos.value.map((td) => line({ t: 'todo', td })),
      ...timeline.value.map((e) => line({ t: 'tl', e: { ...e, full: e.full ? String(e.full).slice(0, 4000) : undefined } })),
      ...trimmed.map((m) => line({ t: 'task', m })),
      ...[...workset.entries()].map(([path, f]) => line({ t: 'file', path, f: { ...f } })),
    ]
    await invoke('ai_session_save', {
      templateId: Number(props.templateId),
      name: sessionName.value,
      data: lines.join('\n'),
    })
  } catch { /* 写盘失败静默,内存态继续 */ }
}

// 从 JSONL 原文恢复全部面板状态(sessionName 由调用方先设好,保证随后的自动保存落到正确文件)
const restoreSession = (raw) => {
  const d = { chat: [], tl: [], task: [], file: [], todo: [], meta: {} }
  for (const l of raw.data.split('\n')) {
    if (!l.trim()) continue
    try { const o = JSON.parse(l); (d[o.t] || (d[o.t] = [])).push(o.m ?? o.e ?? o.f ?? o) } catch {}
  }
  const meta = (d.meta && d.meta[0]) || {}
  todos.value = (d.todo || []).map((x) => x.td || x).filter((x) => x && x.title)
  timeline.value = d.tl || []
  taskMessages.value = d.task || []
  tokIn.value = meta.tokIn || 0
  tokOut.value = meta.tokOut || 0
  appliedInfo.value = null
  workset.clear()
  for (const item of d.file || []) {
    if (item && item.path) workset.set(item.path, { ...item.f, version: 0, readVersion: 0, hasRead: true })
  }
  refreshDirty()
}
const loadSession = async () => {
  try {
    const raw = await invoke('ai_session_load', { templateId: Number(props.templateId) })
    if (!raw || !raw.data) return
    sessionName.value = raw.name.replace('.jsonl', '')
    restoreSession(raw)
  } catch { /* 损坏则丢弃 */ }
}
const clearSession = () => { try { if (sessionName.value) invoke('ai_session_clear', { templateId: Number(props.templateId), name: sessionName.value + '.jsonl' }); sessionName.value = null } catch {} }

// ---- 会话历史列表/切换 ----
const historyOpen = ref(false)
const historyList = ref([])
const historyLoading = ref(false)
const fmtTime = (ms) => {
  const d = new Date(Number(ms))
  return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}
const fmtSize = (b) => (b > 1024 ? `${(b / 1024).toFixed(1)}k` : `${b}B`)
const loadHistory = async () => {
  historyLoading.value = true
  try {
    historyList.value = await invoke('ai_session_list', { templateId: Number(props.templateId) })
  } catch { historyList.value = [] } finally { historyLoading.value = false }
}
const toggleHistory = async () => {
  historyOpen.value = !historyOpen.value
  if (historyOpen.value) await loadHistory()
}
const switchSession = (name) => {
  if (busy.value) return
  const doSwitch = async () => {
    try {
      const raw = await invoke('ai_session_load', { templateId: Number(props.templateId), name })
      if (!raw || !raw.data) return
      sessionName.value = raw.name.replace('.jsonl', '')
      restoreSession(raw)
      historyOpen.value = false
    } catch { message.error('加载会话失败') }
  }
  if (dirtyFiles.value.length > 0) {
    Modal.confirm({
      title: '切换会话将丢弃未应用的修改',
      content: `当前有 ${dirtyFiles.value.length} 个文件的 AI 修改未应用`,
      okText: '丢弃并切换', okType: 'danger', cancelText: '取消',
      onOk: doSwitch,
    })
  } else doSwitch()
}
const removeSession = async (name) => {
  try {
    await invoke('ai_session_clear', { templateId: Number(props.templateId), name })
    if (name === (sessionName.value ? sessionName.value + '.jsonl' : '')) {
      // 删除的是当前会话:连同内存态一起清空
      sessionName.value = null
      taskMessages.value = []
      todos.value = []
      timeline.value = []
      workset.clear()
      dirtyFiles.value = []
      appliedInfo.value = null
      tokIn.value = 0
      tokOut.value = 0
    }
    await loadHistory()
  } catch { message.error('删除会话失败') }
}
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
  { name: 'update_todo', description: '维护任务计划清单(整体替换)', parameters: { type: 'object', properties: { items: { type: 'array', description: '计划项列表', items: { type: 'object', properties: { title: { type: 'string' }, status: { type: 'string', description: 'pending|in_progress|done' } }, required: ['title', 'status'] } } }, required: ['items'] } },
]

// ---- 权限模式驱动的自动应用(autoEdit/auto/full):写操作即时落库,修改前快照兜底 ----
const isAutoApply = () => ['autoEdit', 'auto', 'full'].includes(permMode.value)
const tryAutoApply = async (path) => {
  if (!isAutoApply()) return false
  const f = workset.get(path)
  if (!f) return false
  try {
    if (!appliedInfo.value) {
      try {
        const res = await createRelease(tid(), { changelog: `AI 自动应用前快照 ${new Date().toLocaleString()}` })
        const ver = res?.data?.data?.version
        if (ver) appliedInfo.value = { count: 0, version: ver }
      } catch { /* 快照失败不阻断:自动应用是用户显式选择的模式 */ }
    }
    const isNew = !(fileExistsCache.has(path) || (await fileExistsOnServer(path)))
    if (isNew) {
      const parentPath = path.includes('/') ? path.slice(0, path.lastIndexOf('/')) : ''
      await addTemplateFile({ templateId: tid(), fileName: path.split('/').pop(), parentPath, isDirectory: false })
      fileExistsCache.add(path)
    }
    await editTemplateFile({ templateId: tid(), filePath: path, content: f.content })
    const st = diffStat(f.base, f.content)
    f.base = f.content // 基线前移:后续增量继续可审查/可撤销
    if (appliedInfo.value) {
      appliedInfo.value = {
        ...appliedInfo.value,
        count: (appliedInfo.value.count || 0) + 1,
        added: (appliedInfo.value.added || 0) + st.added,
        removed: (appliedInfo.value.removed || 0) + st.removed,
      }
    }
    refreshDirty()
    if (path === props.currentFilePath) emit('buffer-replace', { path, content: f.content })
    if (isNew) emit('files-updated') // 编辑不改树结构,仅新建需要刷新
    return true
  } catch { return false /* 落库失败回落待审流程 */ }
}

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
      if (await tryAutoApply(args.path)) return '已应用替换并自动落库。'
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
      if (await tryAutoApply(args.path)) return '已插入并自动落库。'
      return '已插入。'
    }
    case 'create_file': {
      if (workset.has(args.path)) return '错误:文件已存在,请用 edit_file。'
      workset.set(args.path, { content: args.content, base: '', version: 0, readVersion: 0, hasRead: true })
      if (await tryAutoApply(args.path)) return '已创建并自动应用到模板。'
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
    case 'update_todo': {
      const items = Array.isArray(args.items) ? args.items : []
      todos.value = items.filter((x) => x && x.title).map((x) => ({ title: String(x.title), status: x.status || 'pending' }))
      return `计划已更新(${todos.value.length} 项)`
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
const diffStat = (base, content) => {
  const o = String(base || '').split('\n')
  const n = String(content || '').split('\n')
  let s = 0
  while (s < o.length && s < n.length && o[s] === n[s]) s += 1
  let e = 0
  while (e < o.length - s && e < n.length - s && o[o.length - 1 - e] === n[n.length - 1 - e]) e += 1
  return { added: n.length - s - e, removed: o.length - s - e, start: s }
}
const refreshDirty = () => {
  const list = []
  for (const [path, f] of workset) {
    if (path === props.currentFilePath && f.content === (props.currentFileContent || '')) continue
    const { added, removed, start } = diffStat(f.base, f.content)
    if (removed === 0 && added === 0) continue
    const now = (f.content || '').split('\n')
    const preview = now.slice(start, start + Math.max(added, Math.min(removed, 4), 1)).slice(0, 12).join('\n')
    list.push({ path, added, removed, preview })
  }
  dirtyFiles.value = list
}
watch(workset, () => nextTick(refreshDirty), { deep: true })
const dirtyAdded = computed(() => dirtyFiles.value.reduce((s, d) => s + d.added, 0))
const dirtyRemoved = computed(() => dirtyFiles.value.reduce((s, d) => s + d.removed, 0))

const abortAgent = () => {
  abortFlag.value = true
  timeline.value.push({ kind: 'error', title: '用户中止' })
}

const runAgent = async (textArg) => {
  const task = (typeof textArg === 'string' ? textArg : agentInput.value).trim()
  if (!task || agentRunning.value) return
  agentInput.value = ''
  agentRunning.value = true
  agentStartAt.value = Date.now()
  nowTick.value = agentStartAt.value
  if (!tickTimer) tickTimer = setInterval(() => { nowTick.value = Date.now() }, 1000)
  abortFlag.value = false
  timeline.value.push({ kind: 'tool', title: '任务', detail: task, full: task })

  // 会话续跑:已有线程则追加任务,否则以系统提示开局
  if (taskMessages.value.length === 0) {
    let system = ''
    try {
      system = await invoke('ai_get_agent_prompt')
    } catch { system = '' }
    const ctx = buildExtraContext()
    taskMessages.value = [
      { role: 'system', content: (system || '') + (ctx ? `\n\n当前编辑上下文:\n${ctx}` : '') + thinkDirective() },
    ]
  }
  taskMessages.value.push({ role: 'user', content: task })

  // 思考指令随级别即时生效:系统消息是会话首条,重写其指令段
  if (taskMessages.value[0]?.role === 'system') {
    const base = String(taskMessages.value[0].content).replace(/\n\n\[思考级别:[^\]]*\][^\n]*/g, '')
    taskMessages.value[0].content = base + thinkDirective()
  }

  try {
    for (let round = 0; round < maxRounds.value; round++) {
      if (abortFlag.value) break
      if (estTokens(taskMessages.value) > CTX_BUDGET * 0.85) {
        const r = await compactContext()
        timeline.value.push({
          kind: 'tool',
          title: r?.text ? '已压缩上下文(模型交接摘要)' : '已修剪上下文',
          detail: `${(estTokens(taskMessages.value) / 1000).toFixed(1)}k tok`,
          full: r?.text,
        })
      }
      const raw = await invoke('ai_agent_turn', { messages: taskMessages.value, tools: TOOLS, ...turnOptions() })
      const res = JSON.parse(raw)
      if (res.usage) {
        tokIn.value += res.usage.input || 0
        tokOut.value += res.usage.output || 0
      }
      if (res.type === 'final') {
        const text = res.text || '(无总结)'
        taskMessages.value.push({ role: 'assistant', content: text })
        timeline.value.push({ kind: 'answer', text })
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
          full: String(out).slice(0, 4000),
        })
      }
    }
    refreshDirty()
  } catch (e) {
    timeline.value.push({ kind: 'error', title: '中止', detail: e.message || String(e) })
  } finally {
    agentRunning.value = false
    abortFlag.value = false
    clearInterval(tickTimer)
    tickTimer = null
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
  let applied = 0
  let added = 0
  let removed = 0
  try {
    // 修改前快照:本会话首个应用前创建一次,作为一键撤销锚点(复用版本管理;失败不阻断应用)
    if (!appliedInfo.value) {
      try {
        const res = await createRelease(tid(), { changelog: `AI 修改前快照 ${new Date().toLocaleString()}` })
        const ver = res?.data?.data?.version
        if (ver) appliedInfo.value = { count: 0, version: ver }
      } catch { /* 版本服务不可用时直接应用 */ }
    }
    for (const d of dirtyFiles.value) {
      const f = workset.get(d.path)
      if (!(await fileExistsOnServer(d.path))) {
        const parentPath = d.path.includes('/') ? d.path.slice(0, d.path.lastIndexOf('/')) : ''
        await addTemplateFile({ templateId: tid(), fileName: d.path.split('/').pop(), parentPath, isDirectory: false })
        fileExistsCache.add(d.path)
      }
      await editTemplateFile({ templateId: tid(), filePath: d.path, content: f.content })
      applied += 1
      added += d.added
      removed += d.removed
    }
    message.success(`已应用 ${applied} 个文件`)
    for (const d of dirtyFiles.value) {
      if (d.path === props.currentFilePath) emit('buffer-replace', { path: d.path, content: workset.get(d.path).content })
    }
    emit('files-updated')
    workset.clear()
    dirtyFiles.value = []
    if (appliedInfo.value) appliedInfo.value = { ...appliedInfo.value, count: applied, added, removed }
    timeline.value.push({
      kind: 'done',
      title: `已应用 ${applied} 个文件`,
      detail: appliedInfo.value ? `修改前快照 ${appliedInfo.value.version},可撤销` : '',
    })
  } catch (e) {
    // 部分应用失败也保留快照锚点,便于整体回滚
    if (applied > 0 && appliedInfo.value) appliedInfo.value = { ...appliedInfo.value, count: applied, added, removed }
    message.error('应用失败: ' + (e.message || e))
  } finally {
    applying.value = false
  }
}

// 一键撤销:回滚到修改前快照,并刷新文件树与当前缓冲区
const undoApply = async () => {
  if (!appliedInfo.value?.version || undoing.value) return
  undoing.value = true
  try {
    await rollbackVersion(tid(), appliedInfo.value.version)
    message.success(`已回滚到快照 ${appliedInfo.value.version}`)
    timeline.value.push({ kind: 'done', title: `已回滚到 ${appliedInfo.value.version}` })
    appliedInfo.value = null
    workset.clear()
    dirtyFiles.value = []
    fileExistsCache.clear()
    if (props.currentFilePath) {
      try {
        const res = await getTemplateFileContent(tid(), props.currentFilePath)
        emit('buffer-replace', { path: props.currentFilePath, content: res.data?.data?.content ?? res.data?.data?.fileContent ?? '' })
      } catch { /* 文件可能已不存在 */ }
    }
    emit('files-updated')
  } catch (e) {
    message.error('回滚失败: ' + (e.message || e))
  } finally {
    undoing.value = false
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
  todos.value = []
  workset.clear()
  dirtyFiles.value = []
  appliedInfo.value = null
  timeline.value = []
  tokIn.value = 0
  tokOut.value = 0
}
// 状态全部声明后恢复会话并挂自动保存(避免 TDZ)
loadSession()
const persistWatch = watch(
  [timeline, tokIn, tokOut, workset, taskMessages, todos],
  scheduleSave,
  { deep: true }
)
onUnmounted(() => { clearTimeout(saveTimer.t); persistWatch.stop(); clearInterval(tickTimer); tickTimer = null })

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
.agent-stream { flex: 1; min-height: 0; overflow-y: auto; padding: 14px 14px 10px; display: flex; flex-direction: column; gap: 12px; }
.ai-welcome { flex: 1; justify-content: center; }
.ai-welcome-icon { color: var(--editor-accent, #16a34a); opacity: 0.7; display: inline-flex; }

.ai-sender-wrap { display: flex; flex-direction: column; gap: 8px; padding: 10px 12px 12px; border-top: 1px solid var(--editor-border, #e0e0e6); flex-shrink: 0; }
.ai-apply-row { display: flex; gap: 8px; }

/* composer 底栏(模型/思考/权限):输入框卡片内部底边(ZCode 式) */
.ai-composer { border: 1px solid var(--editor-border, #e0e0e6); border-radius: 10px; background: var(--editor-panel-bg, #fff); overflow: hidden; transition: border-color 0.15s ease; }
.ai-composer:focus-within { border-color: var(--editor-accent, #16a34a); }
/* antdx Sender 的可视边框在根 boxShadow 与内层继承 border 上,一并剥掉,由卡片统一承载 */
.ai-composer :deep(.ant-sender),
.ai-composer :deep(.ant-sender:focus-within),
.ai-composer :deep(.ant-sender *) {
  border: none !important;
  box-shadow: none !important;
  background: transparent !important;
}
.ai-composer :deep(.ant-sender) { padding: 4px 6px 0 10px; }
.ai-composer-bar { display: flex; align-items: center; gap: 2px; padding: 2px 6px 4px; }
.chip { display: inline-flex; align-items: center; gap: 5px; border: none; background: transparent; padding: 3px 8px; font-size: 11.5px; color: var(--editor-muted, #999); cursor: pointer; white-space: nowrap; overflow: hidden; border-radius: 6px; transition: background-color 0.15s ease; }
.chip:hover { background: var(--editor-inset-bg, #f4f4f2); }
.chip-ico { font-size: 12px; }
.chip-caret { font-size: 9px; opacity: 0.6; }
.chip-model { color: var(--editor-primary, #1b1c1f); font-weight: 500; }
/* 权限档位色彩编码:确认=灰 / 自动编辑=绿 / 自动=蓝 / 完全访问=橙警示 */
.chip-perm-confirm { color: var(--editor-muted, #999); }
.chip-perm-autoEdit { color: var(--editor-accent, #16a34a); }
.chip-perm-auto { color: #2563eb; }
.chip-perm-full { color: #d97706; }
.chip-perm-autoEdit:hover { background: rgba(22, 163, 74, 0.08); }
.chip-perm-auto:hover { background: rgba(37, 99, 235, 0.08); }
.chip-perm-full:hover { background: rgba(217, 119, 6, 0.08); }

/* 弹出选择列表(popover 内容,向上弹出) */
.mp-list { display: flex; flex-direction: column; min-width: 210px; max-height: 300px; overflow-y: auto; }
.mp-group { font-size: 11px; font-weight: 600; color: var(--editor-muted, #999); padding: 8px 10px 4px; text-transform: uppercase; letter-spacing: 0.04em; }
.mp-subgroup { font-size: 11px; color: var(--editor-muted, #999); padding: 4px 10px 2px; }
.mp-item { display: flex; align-items: center; gap: 8px; padding: 5px 10px; font-size: 12px; color: var(--editor-primary, #333); cursor: pointer; border-radius: 6px; }
.mp-item:hover { background: var(--editor-inset-bg, #f4f4f2); }
.mp-item.cur { color: var(--editor-accent, #16a34a); font-weight: 600; }
.mp-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.mp-desc { font-size: 11px; color: var(--editor-muted, #999); margin-left: auto; white-space: nowrap; }
.mp-fn { flex: none; font-size: 9px; font-weight: 600; color: var(--editor-accent, #16a34a); border: 1px solid currentColor; border-radius: 4px; padding: 0 3px; }
.mp-empty { padding: 6px 10px; font-size: 11.5px; color: var(--editor-muted, #999); }

/* 权限模式弹层:两行式条目(档位配色图标+标题/换行说明),固定宽 */
.perm-list { display: flex; flex-direction: column; gap: 2px; width: 236px; }
.perm-item { display: flex; align-items: flex-start; gap: 10px; padding: 8px 10px; border-radius: 8px; cursor: pointer; transition: background-color 0.12s ease; }
.perm-item:hover { background: var(--editor-inset-bg, #f4f4f2); }
.perm-item.cur { background: var(--editor-inset-bg, #f4f4f2); }
.perm-ico { font-size: 15px; margin-top: 1px; flex: none; }
.perm-item.confirm .perm-ico { color: var(--editor-muted, #999); }
.perm-item.autoEdit .perm-ico { color: var(--editor-accent, #16a34a); }
.perm-item.auto .perm-ico { color: #2563eb; }
.perm-item.full .perm-ico { color: #d97706; }
.perm-text { flex: 1; min-width: 0; }
.perm-name { display: flex; align-items: center; font-size: 12.5px; font-weight: 600; color: var(--editor-primary, #1b1c1f); }
.perm-desc { font-size: 11px; line-height: 1.5; color: var(--editor-muted, #999); margin-top: 2px; }
.perm-check { margin-left: auto; font-size: 11px; }
.perm-item.cur .perm-check { color: var(--editor-accent, #16a34a); }

.diff-card { border: 1px solid var(--editor-border, #e0e0e6); border-radius: 8px; overflow: hidden; }
.diffs-head { font-size: 12px; color: var(--editor-primary, #1b1c1f); font-weight: 500; padding: 0 2px; display: flex; align-items: center; gap: 8px; }
.applied-bar { display: flex; justify-content: space-between; align-items: center; gap: 8px; padding: 6px 10px; border: 1px solid var(--editor-border, #e0e0e6); border-left: 3px solid var(--editor-accent, #16a34a); border-radius: 8px; font-size: 12px; color: var(--editor-muted, #666); }
.applied-bar b { color: var(--editor-primary, #333); font-weight: 600; font-family: var(--editor-mono, monospace); }
.applied-plus { color: var(--editor-accent, #16a34a); font-family: var(--editor-mono, monospace); }
.applied-minus { color: #dc2626; font-family: var(--editor-mono, monospace); }
.diff-head { display: flex; justify-content: space-between; align-items: center; padding: 6px 10px; background: var(--editor-inset-bg, #f4f4f2); font-size: 12px; }
.diff-path { font-weight: 500; color: var(--editor-primary, #333); word-break: break-all; }
.diff-count { color: var(--editor-accent, #16a34a); flex: none; }
.diff-body { margin: 0; padding: 8px 10px; font-size: 11px; line-height: 1.5; max-height: 120px; overflow: auto; color: var(--editor-muted, #666); white-space: pre-wrap; word-break: break-all; }

/* 紧凑步骤行(终端式) */
.working-row { font-size: 12px; color: var(--editor-muted, #999); padding: 0 6px; }
.steps { display: flex; flex-direction: column; gap: 2px; }
.step { display: flex; align-items: center; gap: 8px; min-height: 24px; padding: 2px 6px; border-radius: 6px; font-size: 12px; cursor: pointer; color: var(--editor-primary, #1b1c1f); }
.step:hover { background: var(--editor-inset-bg, #f4f4f2); }
.step-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; background: var(--editor-muted, #999); }
.step.done .step-dot { background: var(--editor-accent, #16a34a); }
.step.error .step-dot { background: #dc2626; }
.step-title { font-weight: 500; white-space: nowrap; flex: none; }
.step-detail { color: var(--editor-muted, #999); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; min-width: 0; font-size: 11.5px; }
.step-chev { color: var(--editor-muted, #999); flex: none; font-size: 11px; transition: transform 0.15s ease; }
.step.open .step-chev { transform: rotate(90deg); }
.step-full { margin: 2px 6px 6px 20px; padding: 8px 10px; background: var(--editor-inset-bg, #f4f4f2); border-radius: 6px; font-family: var(--editor-mono, monospace); font-size: 11px; line-height: 1.5; color: var(--editor-muted, #666); white-space: pre-wrap; word-break: break-all; max-height: 260px; overflow: auto; }

/* AI 文本回复(内联在步骤流中) */
.answer { padding: 8px 10px; border-left: 3px solid var(--editor-accent, #16a34a); border-radius: 6px; background: var(--editor-inset-bg, #f4f4f2); font-size: 12.5px; line-height: 1.65; color: var(--editor-primary, #1b1c1f); white-space: pre-wrap; word-break: break-word; }

/* 计划清单 */
.todo-box { display: flex; flex-direction: column; gap: 3px; padding: 8px 10px; border: 1px solid var(--editor-border, #e0e0e6); border-radius: 8px; }
.todo-item { display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--editor-primary, #1b1c1f); }
.todo-item.done .todo-title { text-decoration: line-through; color: var(--editor-muted, #999); }
.todo-dot { color: var(--editor-muted, #999); flex: none; }
.todo-item.in_progress .todo-dot { color: var(--editor-accent, #16a34a); }
.todo-title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* @ 引用补全面板 */
.at-panel { display: flex; flex-wrap: wrap; gap: 4px; padding: 8px; border: 1px solid var(--editor-border, #e0e0e6); border-radius: 8px; background: var(--editor-panel-bg, #fff); }
.at-item { border: none; background: var(--editor-inset-bg, #f4f4f2); border-radius: 6px; padding: 3px 8px; font-size: 11.5px; color: var(--editor-primary, #333); cursor: pointer; font-family: var(--editor-mono, monospace); }
.at-item:hover { color: var(--editor-accent, #16a34a); }

/* 收起态迷你徽标 */
.ai-mini { position: fixed; right: 18px; bottom: 18px; width: 40px; height: 40px; border-radius: 50%; border: 1px solid var(--editor-border, #e0e0e6); background: var(--editor-panel-bg, #fff); box-shadow: 0 4px 14px rgba(0, 0, 0, 0.08); display: flex; align-items: center; justify-content: center; cursor: pointer; color: var(--editor-accent, #16a34a); z-index: 100; transition: transform 0.15s ease; }
.ai-mini:hover { transform: scale(1.06); }
.ai-mini-dot { position: absolute; top: 2px; right: 2px; width: 8px; height: 8px; border-radius: 50%; background: var(--editor-accent, #16a34a); }
.ai-mini-dot.spin { animation: ai-breathe 1.2s ease-in-out infinite; }
@keyframes ai-breathe { 0%, 100% { opacity: 0.35; transform: scale(0.8); } 50% { opacity: 1; transform: scale(1.1); } }

/* 会话历史面板 */
.history-box { margin: 10px 12px 0; border: 1px solid var(--editor-border, #e0e0e6); border-radius: 8px; overflow-y: auto; max-height: 220px; flex-shrink: 0; }
.history-head { display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; background: var(--editor-inset-bg, #f4f4f2); font-size: 12px; font-weight: 600; color: var(--editor-primary, #1b1c1f); position: sticky; top: 0; }
.history-count { font-weight: 400; color: var(--editor-muted, #999); }
.history-empty { padding: 14px; font-size: 12px; color: var(--editor-muted, #999); text-align: center; }
.history-item { display: flex; align-items: center; gap: 8px; padding: 5px 10px; font-size: 12px; cursor: pointer; color: var(--editor-primary, #333); border-left: 2px solid transparent; }
.history-item:hover { background: var(--editor-inset-bg, #f4f4f2); }
.history-item.cur { border-left-color: var(--editor-accent, #16a34a); background: rgba(22, 163, 74, 0.05); }
.history-item.cur .history-time { color: var(--editor-accent, #16a34a); font-weight: 600; }
.history-time { font-family: var(--editor-mono, monospace); }
.history-size { color: var(--editor-muted, #999); font-size: 11px; margin-left: auto; }
.history-del { color: var(--editor-muted, #999); flex: none; padding: 2px; display: inline-flex; font-size: 11px; }
.history-del:hover { color: #dc2626; }
</style>
