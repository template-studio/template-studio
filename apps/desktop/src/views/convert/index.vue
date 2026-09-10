<template>
  <div class="cw-page">
    <header class="cw-head">
      <div class="cw-head-left">
        <a-button type="text" size="small" @click="$router.push('/templates')">
          <template #icon><ArrowLeftOutlined /></template>
        </a-button>
        <span class="cw-title">项目转模板</span>
        <template v-if="ir.source.dir">
          <a-tag class="cw-src">{{ ir.source.source }}</a-tag>
          <a-tag v-if="ir.source.branch">{{ ir.source.branch }}</a-tag>
          <a-tag v-if="ir.source.commit" class="cw-mono">{{ ir.source.commit.slice(0, 7) }}</a-tag>
          <a-tag v-if="ir.source.packId" color="green">{{ ir.source.packId }}</a-tag>
          <a-tag v-if="degraded" color="orange" title="未配置 AI 或调用失败,结果为纯启发式">降级模式</a-tag>
        </template>
      </div>
      <div class="cw-head-right">
        <a-button v-if="ir.source.dir" size="small" :disabled="busy" @click="rerunAnalyze">重新分析</a-button>
        <a-button type="primary" size="small" :disabled="!canStore || busy" :loading="storing" @click="openStore">存储为模板</a-button>
      </div>
    </header>

    <!-- ===== 未克隆:来源输入 + 草稿列表 ===== -->
    <div v-if="!ir.source.dir" class="cw-source">
      <div class="cw-source-card">
        <div class="cw-source-title">从 git 项目转换</div>
        <div class="cw-source-sub">完整克隆到本地镜像后操作,原始仓库只读;转换基于已提交内容(HEAD)</div>
        <a-input v-model:value="srcInput" placeholder="远程仓库 URL(github/gitee/gitlab)或本地 git 项目路径" size="large" @pressEnter="startConvert" />
        <div class="cw-source-row">
          <a-input v-model:value="srcBranch" placeholder="分支(可选,默认主分支)" style="width: 240px" allow-clear />
          <a-button type="primary" :loading="busy" @click="startConvert">开始转换</a-button>
        </div>
        <div v-if="errorMsg" class="cw-error">{{ errorMsg }}</div>
      </div>
      <div class="cw-drafts">
        <div class="cw-drafts-title">转换草稿</div>
        <div v-if="drafts.length === 0" class="cw-drafts-empty">暂无草稿</div>
        <div v-for="d in drafts" :key="d.id" class="cw-draft-item" @click="openDraft(d.id)">
          <span class="cw-draft-name">{{ d.meta?.source || d.id }}</span>
          <span class="cw-draft-time">{{ fmtTime(d.mtimeMs) }}</span>
          <DeleteOutlined class="cw-draft-del" @click.stop="removeDraft(d.id)" />
        </div>
      </div>
    </div>

    <!-- ===== 已克隆:三栏 ===== -->
    <div v-else class="cw-main">
      <!-- 左:文件树 -->
      <aside class="cw-left">
        <div class="cw-panel-head">
          文件 <span class="cw-count">{{ keptCount }}/{{ ir.files.length }}</span>
        </div>
        <div class="cw-tree">
          <div v-for="g in fileGroups" :key="g.dir" class="cw-group">
            <div class="cw-group-head" @click="g.open = !g.open">
              <span class="cw-chev" :class="{ open: g.open }">›</span>
              <span class="cw-group-name">{{ g.dir || '(根目录)' }}</span>
              <span class="cw-count">{{ g.files.length }}</span>
            </div>
            <template v-if="g.open">
              <div v-for="f in g.files" :key="f.path" class="cw-file" :class="f.action" @click="toggleFile(f)">
                <span class="cw-dot"></span>
                <span class="cw-file-name" :title="f.reason">{{ f.base }}</span>
                <span v-if="f.isEntry" class="cw-entry">入口</span>
                <EyeOutlined v-if="f.action === 'keep'" class="cw-eye" title="预览模板化内容" @click.stop="preview(f.path)" />
              </div>
            </template>
          </div>
        </div>
      </aside>

      <!-- 中:管线流 + 预览 -->
      <section class="cw-center">
        <div class="cw-stages">
          <div v-for="(s, i) in stages" :key="i" class="cw-stage" :class="s.state">
            <span class="cw-stage-dot"></span>
            <span>{{ s.label }}</span>
            <span v-if="s.detail" class="cw-stage-detail">{{ s.detail }}</span>
          </div>
        </div>

        <div v-for="(c, i) in ir.conflicts" :key="'c' + i" class="cw-issue conflict">
          <b>冲突</b> {{ c.path }} · {{ c.original || '' }} {{ c.reason }}
        </div>
        <div v-for="(w, i) in ir.warnings" :key="'w' + i" class="cw-issue warn">
          <b>提示</b> {{ w.path }} · {{ w.original }}(预期 {{ w.expected }}/实际 {{ w.actual }})已替换全部合法位置
        </div>
        <div v-for="(e, i) in ir.validationErrors" :key="'e' + i" class="cw-issue conflict">
          <b>渲染失败</b> {{ e.path }} · {{ e.error }}
        </div>

        <div v-if="previewFile" class="cw-preview">
          <div class="cw-preview-head">
            <span class="cw-mono">{{ previewFile.path }}</span>
            <span v-if="previewFile.replaced != null" class="cw-preview-count">替换 {{ previewFile.replaced }} 处</span>
            <a-button type="text" size="small" @click="previewFile = null">
              <template #icon><CloseOutlined /></template>
            </a-button>
          </div>
          <pre class="cw-preview-body">{{ previewFile.content }}</pre>
        </div>
        <div v-else-if="ir.source.dir" class="cw-hint">点击左侧文件预览模板化后的内容;绿点保留 / 红点剔除(点击切换)</div>
      </section>

      <!-- 右:变量表 -->
      <aside class="cw-right">
        <div class="cw-panel-head">
          变量 <span class="cw-count">{{ enabledVars.length }}/{{ ir.variables.length }}</span>
        </div>
        <div class="cw-vars">
          <div v-for="(v, i) in ir.variables" :key="i" class="cw-var" :class="{ off: !v.enabled }">
            <div class="cw-var-top">
              <a-checkbox v-model:checked="v.enabled" size="small" />
              <input v-model="v.name" class="cw-var-name cw-mono" :disabled="!v.enabled" spellcheck="false" />
              <span class="cw-conf" :title="`置信度 ${v.confidence}`" :style="{ color: confColor(v.confidence) }">●</span>
            </div>
            <div class="cw-var-mid">
              <span class="cw-var-sem">{{ v.semantic }}</span>
              <input v-model="v.defaultValue" class="cw-var-def" :disabled="!v.enabled" spellcheck="false" />
            </div>
            <div class="cw-var-occ" :title="occTooltip(v)">{{ v.occurrenceCount || 0 }} 处 · {{ v.occurrences?.length || 0 }} 文件</div>
          </div>
        </div>
      </aside>
    </div>

    <!-- 存储对话框 -->
    <a-modal v-model:open="storeOpen" title="存储为模板" ok-text="创建" cancel-text="取消" :confirm-loading="storing" @ok="doStore">
      <a-form layout="vertical">
        <a-form-item label="模板名称" required>
          <a-input v-model:value="storeForm.name" placeholder="如:Go Web 服务脚手架" />
        </a-form-item>
        <a-form-item label="描述">
          <a-input v-model:value="storeForm.description" placeholder="一句话说明用途" />
        </a-form-item>
        <div class="cw-store-sum">将创建 {{ keptCount }} 个文件、{{ enabledVars.length }} 个变量,并自动发布首个版本</div>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import { ArrowLeftOutlined, DeleteOutlined, CloseOutlined, EyeOutlined } from '@ant-design/icons-vue'
import { createUserTemplate } from '@/api/editor/templates/contribution'
import { analyzeTemplateVariables } from '@/api/editor/templates'
import { addTemplateFile, editTemplateFile } from '@/api/editor/templateFiles'
import { createRelease } from '@/api/editor/releases'

const router = useRouter()
const route = useRoute()

// ---- 状态(IR 即工作态,自动落草稿) ----
const ir = reactive({
  source: { source: '', branch: '', commit: '', dir: '', packId: '' },
  files: [],       // [{path, base, dir, action, reason, isEntry, size}]
  variables: [],   // analyze 结果 + enabled 编辑态
  conflicts: [], warnings: [], validationErrors: [],
  outputs: [],
})
const srcInput = ref('')
const srcBranch = ref('')
const busy = ref(false)
const storing = ref(false)
const errorMsg = ref('')
const degraded = ref(false)
const previewFile = ref(null)
const storeOpen = ref(false)
const storeForm = reactive({ name: '', description: '' })
const drafts = ref([])
const draftId = ref('')
const stageState = ref({ clone: '', scan: '', analyze: '' })

const tid = () => draftId.value
const keptCount = computed(() => ir.files.filter((f) => f.action === 'keep').length)
const enabledVars = computed(() => ir.variables.filter((v) => v.enabled))
const canStore = computed(() => ir.source.dir && keptCount.value > 0 && !busy.value)

const stages = computed(() => [
  { label: '克隆', state: stageState.value.clone, detail: ir.source.commit ? ir.source.commit.slice(0, 7) : '' },
  { label: '扫描', state: stageState.value.scan, detail: ir.files.length ? `${keptCount.value}/${ir.files.length} 保留` : '' },
  { label: '分析', state: stageState.value.analyze, detail: ir.variables.length ? `${ir.variables.length} 候选变量` : '' },
  { label: '存储', state: '', detail: '' },
])

// ---- 文件分组(左栏) ----
const fileGroups = computed(() => {
  const map = new Map()
  for (const f of ir.files) {
    const dir = f.path.includes('/') ? f.path.slice(0, f.path.lastIndexOf('/')) : ''
    if (!map.has(dir)) map.set(dir, { dir, open: true, files: [] })
    map.get(dir).files.push(f)
  }
  return [...map.values()]
})
const toggleFile = (f) => {
  f.action = f.action === 'keep' ? 'exclude' : 'keep'
  if (f.action === 'exclude') f.reason = '手动剔除'
  else f.reason = ''
}

// ---- 管线 ----
const startConvert = async () => {
  const src = srcInput.value.trim()
  if (!src || busy.value) return
  busy.value = true
  errorMsg.value = ''
  try {
    stageState.value = { clone: 'run', scan: '', analyze: '' }
    const raw = await invoke('convert_clone', { source: src, branch: srcBranch.value.trim() || null })
    const c = JSON.parse(raw)
    draftId.value = String(Date.now())
    Object.assign(ir.source, { source: src, branch: c.branch, commit: c.commit, dir: c.dir })
    stageState.value.clone = 'done'
    await doScan()
  } catch (e) {
    errorMsg.value = String(e)
    stageState.value.clone = 'error'
  } finally {
    busy.value = false
  }
}

const doScan = async () => {
  stageState.value.scan = 'run'
  ir.files = []
  ir.variables = []
  const raw = await invoke('convert_scan', { root: ir.source.dir })
  const s = JSON.parse(raw)
  ir.source.packId = s.packId
  ir.files = s.files.map((f) => ({
    ...f,
    base: f.path.split('/').pop(),
    action: f.action === 'exclude' ? 'exclude' : 'keep',
  }))
  stageState.value.scan = 'done'
  await doAnalyze()
}

const doAnalyze = async () => {
  stageState.value.analyze = 'run'
  const keeps = ir.files.filter((f) => f.action === 'keep').map((f) => f.path)
  try {
    const raw = await invoke('convert_analyze', { root: ir.source.dir, files: keeps, provider: null, model: null, thinking: null })
    const a = JSON.parse(raw)
    degraded.value = !!a.degraded
    // AI 文件分类渐进落树(仅影响 AI 建议剔除且当前为 keep 的文件)
    for (const fc of a.fileClasses || []) {
      if (fc.action !== 'exclude') continue
      const f = ir.files.find((x) => x.path === fc.path && x.action === 'keep' && !x.isEntry)
      if (f) { f.action = 'exclude'; f.reason = `AI: ${fc.reason || '业务代码'}` }
    }
    ir.variables = (a.variables || []).map((v) => ({
      ...v,
      enabled: (v.confidence ?? 0) >= 0.6 || v.occurrenceCount > 2,
    }))
    stageState.value.analyze = 'done'
  } catch (e) {
    degraded.value = true
    stageState.value.analyze = 'error'
    message.error('分析失败: ' + (e.message || e))
  }
}
const rerunAnalyze = () => { if (!busy.value) { busy.value = true; doAnalyze().finally(() => { busy.value = false }) } }

// ---- 预览(应用替换结果) ----
const runApply = async () => {
  const keeps = ir.files.filter((f) => f.action === 'keep').map((f) => f.path)
  const vars = enabledVars.value.map((v) => ({
    name: v.name.trim(), defaultValue: v.defaultValue,
    occurrences: v.occurrences,
  }))
  const raw = await invoke('convert_apply', { root: ir.source.dir, files: keeps, variables: vars })
  const r = JSON.parse(raw)
  ir.conflicts = r.conflicts || []
  ir.warnings = r.warnings || []
  ir.validationErrors = r.validationErrors || []
  ir.outputs = r.outputs || []
  return r
}

// 预览:无替换结果时先试运行 apply,再取该文件模板化内容
const preview = async (path) => {
  try {
    if (!ir.outputs.length) await runApply()
    const o = ir.outputs.find((x) => x.path === path)
    if (!o) { message.info('该文件无模板化内容'); return }
    previewFile.value = o
  } catch (e) { message.error('预览失败: ' + (e.message || e)) }
}
// 状态变化使替换结果失效
watch(() => ir.files.map((f) => f.path + f.action).join('|') + JSON.stringify(enabledVars.value.map((v) => [v.name, v.defaultValue])), () => {
  previewFile.value = null
  ir.outputs = []
  ir.conflicts = []; ir.warnings = []; ir.validationErrors = []
})

// ---- 存储 ----
const openStore = async () => {
  busy.value = true
  try {
    const r = await runApply()
    if (!r.clean) {
      message.warning('存在冲突或渲染失败,已在中栏列出,请处理后重试')
      return
    }
    if (!storeForm.name) storeForm.name = ir.source.source.split('/').pop()?.replace(/\.git$/, '') || '转换模板'
    storeOpen.value = true
  } catch (e) {
    message.error('替换失败: ' + (e.message || e))
  } finally {
    busy.value = false
  }
}

const doStore = async () => {
  if (!storeForm.name.trim()) { message.error('请填写模板名称'); return }
  storing.value = true
  try {
    const outputs = ir.outputs.length ? ir.outputs : (await runApply()).outputs
    const res = await createUserTemplate({
      name: storeForm.name.trim(), templateType: 'default', categoryId: null,
      description: storeForm.description.trim(), visibility: 'private', languages: [],
    })
    const templateId = res?.data?.data?.id
    if (!templateId) throw new Error('创建模板失败:未返回模板 ID')
    // 目录先行(深度排序),再写文件
    const dirSet = new Set()
    for (const f of outputs) {
      const parts = f.path.split('/')
      parts.pop()
      for (let i = 1; i <= parts.length; i++) dirSet.add(parts.slice(0, i).join('/'))
    }
    for (const dir of [...dirSet].sort((a, b) => a.split('/').length - b.split('/').length)) {
      await addTemplateFile({ templateId, fileName: dir.split('/').pop(), parentPath: dir.includes('/') ? dir.slice(0, dir.lastIndexOf('/')) : '', isDirectory: true })
    }
    for (const f of outputs) {
      await addTemplateFile({ templateId, fileName: f.path.split('/').pop(), parentPath: f.path.includes('/') ? f.path.slice(0, f.path.lastIndexOf('/')) : '', isDirectory: false })
      await editTemplateFile({ templateId, filePath: f.path, content: f.content })
    }
    // 占位符注册为变量(类型/默认值细化在编辑器变量设计器完成)
    try { await analyzeTemplateVariables(templateId) } catch { /* 不阻断 */ }
    try { await createRelease(templateId, { changelog: '项目转换初始版本' }) } catch { /* 版本失败不阻断 */ }
    message.success(`模板创建成功(${outputs.length} 文件)`)
    if (tid()) { try { await invoke('convert_draft_delete', { id: tid() }) } catch {} }
    router.push(`/editor/${templateId}`)
  } catch (e) {
    message.error('创建失败: ' + (e.message || e))
  } finally {
    storing.value = false
  }
}

// ---- 草稿持久化(防抖自动保存) ----
let saveTimer = null
const scheduleSave = () => {
  if (!tid()) return
  clearTimeout(saveTimer)
  saveTimer = setTimeout(async () => {
    try {
      await invoke('convert_draft_save', {
        id: tid(),
        meta: { source: ir.source, degraded: degraded.value, savedAt: Date.now() },
        ir: { files: ir.files.map(({ path, action, reason }) => ({ path, action, reason })), variables: ir.variables },
      })
    } catch { /* 静默 */ }
  }, 800)
}
watch([() => ir.files.map((f) => f.path + f.action).join('|'), () => JSON.stringify(ir.variables)], scheduleSave)

const loadDrafts = async () => {
  try {
    const raw = await invoke('convert_draft_list')
    drafts.value = JSON.parse(raw).items || []
  } catch { drafts.value = [] }
}
const openDraft = async (id) => {
  try {
    const raw = await invoke('convert_draft_load', { id })
    const d = JSON.parse(raw)
    if (!d.meta?.source?.dir) { message.error('草稿损坏'); return }
    draftId.value = id
    Object.assign(ir.source, d.meta.source)
    degraded.value = !!d.meta.degraded
    ir.files = (d.ir?.files || []).map((f) => ({ ...f, base: f.path.split('/').pop() }))
    ir.variables = d.ir?.variables || []
    stageState.value = { clone: 'done', scan: 'done', analyze: 'done' }
  } catch (e) { message.error('打开草稿失败: ' + (e.message || e)) }
}
const removeDraft = async (id) => {
  try { await invoke('convert_draft_delete', { id }); await loadDrafts() } catch {}
}

const fmtTime = (ms) => {
  const d = new Date(Number(ms))
  return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}
const confColor = (c) => (c >= 0.7 ? 'var(--color-brand, #16a34a)' : c >= 0.5 ? '#d97706' : '#999')
const occTooltip = (v) => (v.occurrences || []).map((o) => `${o.path} ×${o.count}`).join('\n')

onMounted(() => {
  loadDrafts()
  if (route.query.draft) openDraft(String(route.query.draft))
})
</script>

<style scoped>
.cw-page { height: 100%; display: flex; flex-direction: column; background: var(--color-canvas, #f1f1ee); }
.cw-head { display: flex; align-items: center; justify-content: space-between; padding: 10px 16px; background: var(--color-background, #fff); border-bottom: 1px solid var(--color-border, #e5e5e2); flex-shrink: 0; }
.cw-head-left { display: flex; align-items: center; gap: 8px; min-width: 0; }
.cw-title { font-size: 14px; font-weight: 600; color: var(--color-text, #1b1c1f); margin-right: 6px; }
.cw-src { max-width: 340px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-mono { font-family: Consolas, 'JetBrains Mono', monospace; }
.cw-head-right { display: flex; gap: 8px; }

.cw-source { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 24px; padding: 24px; }
.cw-source-card { width: min(640px, 100%); background: var(--color-background, #fff); border-radius: 12px; padding: 24px; box-shadow: var(--shadow-panel, 0 1px 3px rgba(0,0,0,0.06)); display: flex; flex-direction: column; gap: 12px; }
.cw-source-title { font-size: 16px; font-weight: 600; color: var(--color-text, #1b1c1f); }
.cw-source-sub { font-size: 12px; color: var(--color-text-secondary, #999); }
.cw-source-row { display: flex; gap: 8px; }
.cw-error { color: #dc2626; font-size: 12px; white-space: pre-wrap; }
.cw-drafts { width: min(640px, 100%); }
.cw-drafts-title { font-size: 12px; font-weight: 600; color: var(--color-text-secondary, #999); margin-bottom: 6px; }
.cw-drafts-empty { font-size: 12px; color: var(--color-text-secondary, #999); padding: 8px 0; }
.cw-draft-item { display: flex; align-items: center; gap: 10px; padding: 8px 12px; background: var(--color-background, #fff); border-radius: 8px; margin-bottom: 4px; cursor: pointer; font-size: 13px; }
.cw-draft-item:hover { background: var(--color-surface, #f7f7f5); }
.cw-draft-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-draft-time { font-size: 11px; color: var(--color-text-secondary, #999); }
.cw-draft-del { color: var(--color-text-secondary, #999); font-size: 12px; }
.cw-draft-del:hover { color: #dc2626; }

.cw-main { flex: 1; min-height: 0; display: flex; }
.cw-left, .cw-right { width: 280px; flex-shrink: 0; background: var(--color-background, #fff); display: flex; flex-direction: column; min-height: 0; }
.cw-left { border-right: 1px solid var(--color-border, #e5e5e2); }
.cw-right { border-left: 1px solid var(--color-border, #e5e5e2); }
.cw-panel-head { padding: 10px 12px 6px; font-size: 12px; font-weight: 600; color: var(--color-text, #1b1c1f); flex-shrink: 0; display: flex; gap: 6px; align-items: center; }
.cw-count { font-weight: 400; color: var(--color-text-secondary, #999); }
.cw-tree, .cw-vars { flex: 1; min-height: 0; overflow-y: auto; padding: 0 8px 12px; }
.cw-group-head { display: flex; align-items: center; gap: 6px; padding: 4px 6px; font-size: 11.5px; color: var(--color-text-secondary, #999); cursor: pointer; border-radius: 6px; }
.cw-group-head:hover { background: var(--color-surface, #f7f7f5); }
.cw-chev { display: inline-block; transition: transform 0.15s ease; }
.cw-chev.open { transform: rotate(90deg); }
.cw-group-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: rtl; text-align: left; }
.cw-file { display: flex; align-items: center; gap: 8px; padding: 3px 6px 3px 18px; font-size: 12px; border-radius: 6px; cursor: pointer; color: var(--color-text, #333); }
.cw-file:hover { background: var(--color-surface, #f7f7f5); }
.cw-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; background: var(--color-brand, #16a34a); }
.cw-file.exclude .cw-dot { background: #dc2626; }
.cw-file.exclude .cw-file-name { color: var(--color-text-secondary, #999); text-decoration: line-through; }
.cw-file-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-eye { flex: none; font-size: 11px; color: var(--color-text-secondary, #999); }
.cw-eye:hover { color: var(--color-brand, #16a34a); }
.cw-entry { flex: none; font-size: 9px; color: var(--color-brand, #16a34a); border: 1px solid currentColor; border-radius: 4px; padding: 0 3px; }

.cw-center { flex: 1; min-width: 0; padding: 16px 18px; overflow-y: auto; display: flex; flex-direction: column; gap: 12px; }
.cw-stages { display: flex; flex-direction: column; gap: 2px; }
.cw-stage { display: flex; align-items: center; gap: 8px; min-height: 24px; font-size: 12px; padding: 2px 6px; border-radius: 6px; }
.cw-stage-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--color-border, #ddd); }
.cw-stage.done .cw-stage-dot { background: var(--color-brand, #16a34a); }
.cw-stage.run .cw-stage-dot { background: #d97706; animation: cw-breathe 1.2s ease-in-out infinite; }
.cw-stage.error .cw-stage-dot { background: #dc2626; }
.cw-stage-detail { color: var(--color-text-secondary, #999); font-size: 11.5px; }
@keyframes cw-breathe { 0%, 100% { opacity: 0.35; } 50% { opacity: 1; } }
.cw-issue { font-size: 12px; padding: 6px 10px; border-radius: 8px; }
.cw-issue b { margin-right: 6px; }
.cw-issue.conflict { background: rgba(220, 38, 38, 0.06); color: #b91c1c; }
.cw-issue.warn { background: rgba(217, 119, 6, 0.06); color: #b45309; }
.cw-preview { display: flex; flex-direction: column; border: 1px solid var(--color-border, #e5e5e2); border-radius: 8px; overflow: hidden; }
.cw-preview-head { display: flex; align-items: center; gap: 10px; padding: 6px 10px; background: var(--color-surface, #f7f7f5); font-size: 12px; }
.cw-preview-count { color: var(--color-brand, #16a34a); }
.cw-preview-head button { margin-left: auto; }
.cw-preview-body { margin: 0; padding: 10px 12px; font-size: 11.5px; line-height: 1.55; font-family: Consolas, 'JetBrains Mono', monospace; max-height: 420px; overflow: auto; white-space: pre-wrap; word-break: break-all; color: var(--color-text, #333); }
.cw-hint { font-size: 12px; color: var(--color-text-secondary, #999); padding: 8px; }

.cw-var { border: 1px solid var(--color-border, #e5e5e2); border-radius: 8px; padding: 8px 10px; margin-bottom: 6px; display: flex; flex-direction: column; gap: 4px; }
.cw-var.off { opacity: 0.55; }
.cw-var-top { display: flex; align-items: center; gap: 6px; }
.cw-var-name { flex: 1; min-width: 0; border: none; outline: none; font-size: 12.5px; font-weight: 600; color: var(--color-text, #1b1c1f); background: transparent; }
.cw-var-name:focus { border-bottom: 1px solid var(--color-brand, #16a34a); }
.cw-conf { font-size: 10px; flex: none; }
.cw-var-mid { display: flex; align-items: center; gap: 8px; }
.cw-var-sem { flex: none; font-size: 10px; color: var(--color-text-secondary, #999); border: 1px solid var(--color-border, #e5e5e2); border-radius: 4px; padding: 0 4px; }
.cw-var-def { flex: 1; min-width: 0; border: none; outline: none; font-size: 12px; color: var(--color-text-secondary, #666); background: transparent; font-family: Consolas, monospace; }
.cw-var-def:focus { border-bottom: 1px solid var(--color-brand, #16a34a); }
.cw-var-occ { font-size: 10.5px; color: var(--color-text-secondary, #999); }
.cw-store-sum { font-size: 12px; color: var(--color-text-secondary, #999); }
</style>
