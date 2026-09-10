<template>
  <div class="cw-page">
    <!-- ===== 顶栏(类编辑器 EditHeader) ===== -->
    <header class="cw-head">
      <div class="cw-head-left">
        <button class="cw-icon-btn" title="返回引导页" @click="$router.push('/convert')">
          <ArrowLeftOutlined />
        </button>
        <span class="cw-title">模板转换</span>
        <template v-if="ir.source.dir">
          <span class="cw-vbar"></span>
          <a-tag :color="tplType === 'data_driven' ? 'blue' : 'green'" style="margin-inline-end: 0">{{ tplType === 'data_driven' ? '数据驱动' : '脚手架' }}</a-tag>
          <span class="cw-src cw-mono" :title="ir.source.source">{{ srcShort }}</span>
          <a-tag v-if="ir.source.branch" style="margin-inline-end: 0">{{ ir.source.branch }}</a-tag>
          <a-tag v-if="ir.source.commit" class="cw-mono" style="margin-inline-end: 0">{{ ir.source.commit.slice(0, 7) }}</a-tag>
          <a-tag v-if="ir.source.packId" color="green" style="margin-inline-end: 0">{{ ir.source.packId }}</a-tag>
          <a-tag v-if="degraded" color="orange" title="未配置 AI 或调用失败,结果为纯启发式" style="margin-inline-end: 0">降级模式</a-tag>
        </template>
      </div>
      <div class="cw-head-right">
        <a-button v-if="ir.source.dir" size="small" :class="{ 'cw-ai-on': aiDock }" :title="aiDock ? '收起转换助手' : '打开转换助手(实时过程+调整方向)'" @click="aiDock = !aiDock">
          <template #icon><AiIcon :size="14" /></template>助手
        </a-button>
        <a-button v-if="ir.source.dir" size="small" :disabled="busy" @click="rerunAnalyze">
          <template #icon><RedoOutlined /></template>重新分析
        </a-button>
        <a-button type="primary" size="small" :disabled="!canStore || busy" :loading="storing" @click="openStore">
          <template #icon><SaveOutlined /></template>存储为模板
        </a-button>
        <!-- 窗口控制(独立全屏页自带,与编辑器一致) -->
        <div class="cw-win">
          <button class="cw-icon-btn" title="最小化" @click="winMin"><MinusOutlined /></button>
          <button class="cw-icon-btn" title="最大化/还原" @click="winMax"><BorderOutlined /></button>
          <button class="cw-icon-btn danger" title="关闭窗口" @click="winClose"><CloseOutlined /></button>
        </div>
      </div>
    </header>

    <!-- ===== 初始运行态:来源由引导页/新建弹窗传入,克隆中显示实时进度 ===== -->
    <div v-if="!ir.source.dir" class="cw-source">
      <div class="cw-source-card">
        <div class="cw-source-title">{{ stageState.clone === 'error' ? '引入项目失败' : '正在引入项目…' }}</div>
        <div class="cw-src-now cw-mono" :title="lastSrc">{{ lastSrc }}</div>
        <div class="cw-run-stages">
          <div v-for="(s, i) in stages" :key="i" class="cw-rstage" :class="s.state">
            <span class="cw-rdot"></span>{{ s.label }}
          </div>
        </div>
        <div v-if="errorMsg" class="cw-error">{{ errorMsg }}</div>
        <div class="cw-run-log" ref="runLogEl">
          <div v-for="(a, i) in cloneTail" :key="i" class="cw-run-line"><span class="cw-run-ts">{{ fmtTs(a.ts) }}</span>{{ a.text }}</div>
          <div v-if="cloneTail.length === 0" class="cw-run-line dim">等待 git 输出…</div>
        </div>
        <a-button v-if="stageState.clone === 'error'" size="small" style="align-self: flex-start" @click="$router.replace('/convert')">返回引导页重试</a-button>
      </div>
    </div>

    <!-- ===== 工作台三栏 ===== -->
    <div v-else class="cw-main">
      <!-- 中:文件预览 / 转换过程 -->
      <!-- Activity Bar:分区切换(VSCode 式窄竖条) -->
      <nav class="cw-actbar">
        <button class="cw-act-item" :class="{ on: activeView === 'source' }" title="源代码区:镜像源文件与保留/剔除" @click="activeView = 'source'">
          <FolderFilled class="cw-act-ico" />
        </button>
        <button class="cw-act-item tpl" :class="{ on: activeView === 'template' }" title="模板区:模板文件集与渲染" @click="activeView = 'template'">
          <FileTextFilled class="cw-act-ico" />
        </button>
        <button v-if="isDataDriven" class="cw-act-item focus" :class="{ on: activeView === 'focus' }" title="重点文件:勾选与 AI 暴露策略" @click="activeView = 'focus'">
          <StarFilled class="cw-act-ico" />
          <span v-if="focusFiles.length" class="cw-act-n">{{ focusFiles.length }}</span>
        </button>
        <button class="cw-act-item proc" :class="{ on: bottomOpen }" title="转换过程:时间线与问题" @click="bottomOpen = !bottomOpen">
          <ClockCircleFilled class="cw-act-ico" />
          <span v-if="pipelineRunning" class="cw-act-live"></span>
        </button>
        <button class="cw-act-item vars" :class="{ on: activeView === 'vars' }" title="变量:候选变量启停与默认值" @click="activeView = 'vars'">
          <VariableIcon :size="18" class="cw-act-ico" />
          <span v-if="ir.variables.length" class="cw-act-n">{{ ir.variables.length }}</span>
        </button>
      </nav>

      <!-- Side Bar:当前分区视图 -->
      <aside v-if="activeView !== 'focus'" class="cw-sidebar">
        <template v-if="activeView === 'source'">
          <div class="cw-panel-head">
            <span>源代码</span>
            <span class="cw-count">{{ keptCount }}/{{ ir.files.length }} 保留</span>
          </div>
        <div class="cw-tree">
          <a-tree
            :tree-data="treeData"
            :selected-keys="selectedPath ? [selectedPath] : []"
            :expanded-keys="expandedKeys"
            :field-names="{ key: 'key', title: 'title', children: 'children' }"
            @select="onTreeSelect"
            @expand="onTreeExpand"
          >
            <template #icon="{ data }">
              <FolderOpenFilled v-if="data.isDir && expandedSet.has(String(data.key))" class="cw-fic dir" />
              <FolderFilled v-else-if="data.isDir" class="cw-fic dir" />
              <FileOutlined v-else class="cw-fic" />
            </template>
            <template #title="opt">
              <div
                class="cw-row"
                :class="{ excluded: opt.file && opt.file.action === 'exclude' }"
                :title="opt.file?.reason || opt.title"
              >
                <template v-if="!opt.isDir">
                  <span class="cw-dot" :class="opt.file?.action"></span>
                  <span class="cw-name">{{ opt.title }}</span>
                  <span v-if="opt.file?.isEntry" class="cw-entry">入口</span>
                  <button
                    class="cw-toggle"
                    :class="{ off: opt.file?.action === 'exclude' }"
                    @click.stop="toggleFile(opt.file)"
                  >{{ opt.file?.action === 'exclude' ? '剔除' : '保留' }}</button>
                </template>
                <template v-else>
                  <span class="cw-name">{{ opt.title }}</span>
                  <span class="cw-dircount">{{ dirKept(opt.key) }}/{{ filesUnder(opt.key).length }}</span>
                  <button class="cw-toggle" @click.stop="toggleDir(opt.key)">
                    {{ dirAllExcluded(opt.key) ? '全留' : '全剔' }}
                  </button>
                </template>
              </div>
            </template>
          </a-tree>
        </div>
      
        </template>
        <template v-else-if="activeView === 'template'">
          <div class="cw-panel-head">
            <span>模板文件</span>
            <span class="cw-count">{{ keptFiles.length }} 文件</span>
          </div>
          <div class="cw-tree">
            <a-tree
              :tree-data="tplTreeData"
              :selected-keys="selectedPath ? [selectedPath] : []"
              :expanded-keys="expandedKeys"
              :field-names="{ key: 'key', title: 'title', children: 'children' }"
              @select="onTplTreeSelect"
              @expand="onTreeExpand"
            >
              <template #title="opt">
                <div class="cw-row" :title="opt.title">
                  <template v-if="!opt.isDir">
                    <span class="cw-name">{{ opt.title }}</span>
                    <span v-if="opt.file?.isEntry" class="cw-entry">入口</span>
                    <span v-if="tplReplacedOf(opt.key)" class="cw-ra-badge">{{ tplReplacedOf(opt.key) }}</span>
                  </template>
                  <template v-else>
                    <span class="cw-name">{{ opt.title }}</span>
                  </template>
                </div>
              </template>
            </a-tree>
            <div v-if="!keptFiles.length" class="cw-tl-empty">模板暂无文件——在源代码区切换保留</div>
          </div>
        </template>
        <template v-else-if="activeView === 'vars'">
          <div class="cw-panel-head">
            <span>变量</span>
            <span class="cw-count">{{ enabledVars.length }}/{{ ir.variables.length }} 启用</span>
          </div>
          <div class="cw-vars">
            <div v-if="ir.variables.length === 0" class="cw-tl-empty">尚无候选变量——完成分析后在此调整</div>
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
        </template>
      </aside>

      <div class="cw-center-col">
            <section class="cw-center">
        <div class="cw-chead">
          <span class="cw-crumb cw-mono" :title="selectedPath">{{ selectedPath || activeLabel }}</span>
          <div class="cw-chead-right">
            <template v-if="selectedPath">
              <button class="cw-icon-btn sm" title="关闭预览" @click="closePreview"><CloseOutlined /></button>
              <div class="cw-seg">
                <button :class="{ on: previewMode === 'render' }" title="变量默认值注入后的生成效果(编辑器同款渲染引擎)" @click="previewMode = 'render'">渲染</button>
                <button :class="{ on: previewMode === 'tpl' }" @click="previewMode = 'tpl'">模板化</button>
                <button :class="{ on: previewMode === 'raw' }" @click="previewMode = 'raw'">原文</button>
              </div>
              <span v-if="previewReplaced" class="cw-repl">替换 {{ previewReplaced }} 处</span>
            </template>
            <a-button v-if="ir.source.dir" size="small" :loading="buildChecking" title="渲染落盘后按技术栈构建命令冒烟验证(存储前可选门禁)" @click="runBuildCheck">
              <template #icon><BuildOutlined /></template>构建验证
            </a-button>
          </div>
        </div>

        <!-- 重点文件工作台(主内容区:左勾选树 + 右策略面板) -->
        <div v-if="activeView === 'focus'" class="cw-focus-main">
          <div class="cw-focus-treebox">
            <div class="cw-focus-tree">
              <a-tree
                v-model:checkedKeys="focusChecked"
                checkable
                :tree-data="treeData"
                :selected-keys="selectedPath ? [selectedPath] : []"
                :expanded-keys="expandedKeys"
                :field-names="{ key: 'key', title: 'title', children: 'children' }"
                @select="onTreeSelect"
                @expand="onTreeExpand"
              >
                <template #title="opt">
                  <div class="cw-row" :class="{ excluded: opt.file && opt.file.action === 'exclude' }" :title="opt.file?.reason || opt.title">
                    <template v-if="!opt.isDir">
                      <span class="cw-dot" :class="opt.file?.action"></span>
                      <span class="cw-name">{{ opt.title }}</span>
                      <span v-if="opt.file?.isEntry" class="cw-entry">入口</span>
                    </template>
                    <template v-else>
                      <span class="cw-name">{{ opt.title }}</span>
                    </template>
                  </div>
                </template>
              </a-tree>
            </div>
          </div>
          <div class="cw-focus-side">
            <div class="cw-panel-head">
              <span>重点文件</span>
              <span class="cw-count">已勾选 {{ focusFiles.length }}</span>
            </div>
            <div class="cw-focus-desc">
              <b>勾选重点文件</b>——以文件为最小单元:例如 controller → service → mapper 的一条链路,或某个交互流程(树形菜单等)涉及的文件。
            </div>
            <a-button v-if="stageState.analyze === 'wait'" type="primary" size="small" @click="rerunAnalyze">下一步 · 开始分析</a-button>
            <a-button v-else-if="focusDirty" size="small" @click="rerunAnalyze">重新分析以应用勾选</a-button>
            <div class="cw-focus-opts">
              <div class="cw-focus-opt-label">其它文件对 AI:</div>
              <a-radio-group v-model:value="exposeAll">
                <a-radio :value="true">全部暴露<small>内容都提供给 AI,细节更全,更耗 token</small></a-radio>
                <a-radio :value="false">聚焦勾选<small>仅勾选文件内容 + 全项目目录结构,其它文件对 AI 不可见</small></a-radio>
              </a-radio-group>
            </div>
            <span class="cw-focus-count">已勾选 {{ focusFiles.length }} 个文件</span>
          </div>
        </div>

        <!-- 文件内容区 -->
        <div v-else-if="selectedPath" class="cw-filepane">
          <div v-if="!selectedPath" class="cw-empty">点击左侧文件查看预览;行尾「保留/剔除」控制该文件是否进入模板</div>
          <template v-else>
            <div v-if="tplStale" class="cw-stale">
              变量或文件清单已变更,模板化预览已失效
              <a-button size="small" type="link" @click="regenPreview">重新生成</a-button>
            </div>
            <div v-if="renderError" class="cw-issue conflict"><b>渲染失败</b>{{ renderError }}</div>
            <div v-if="previewError" class="cw-issue warn"><b>无法读取</b>{{ previewError }}</div>
            <CodeViewer v-if="previewMode === 'render' && renderedContent && !renderError" :content="renderedContent" :filename="selectedPath" class="cw-code-cm" />
            <div v-else class="cw-code">
              <div v-for="(l, i) in previewLines" :key="i" class="cw-ln">
                <span class="cw-no">{{ i + 1 }}</span>
                <span class="cw-lc" v-html="l"></span>
              </div>
            </div>
          </template>
        </div>
        <div v-else class="cw-filepane"><div class="cw-empty">{{ emptyHint }}</div></div>
      </section>

        <!-- 底部面板:转换过程 -->
        <div v-if="bottomOpen" class="cw-bottom">
          <div class="cw-bottom-head">
            <span>转换过程</span>
            <a-button v-if="stageState.analyze === 'wait'" type="primary" size="small" @click="rerunAnalyze">下一步 · 开始分析</a-button>
            <a-button v-else-if="focusDirty" size="small" @click="rerunAnalyze">重新分析以应用勾选</a-button>
            <button class="cw-icon-btn sm" title="收起" @click="bottomOpen = false"><CloseOutlined /></button>
          </div>
                    <div class="cw-stagestrip">
            <template v-for="(s, i) in stages" :key="i">
              <div class="cw-pstage" :class="s.state">
                <span class="cw-pdot"></span>
                <span>{{ s.label }}</span>
                <span v-if="s.detail" class="cw-pdetail">{{ s.detail }}</span>
              </div>
              <span v-if="i < stages.length - 1" class="cw-plink"></span>
            </template>
          </div>

          <div v-for="(c, i) in ir.conflicts" :key="'c' + i" class="cw-issue conflict"><b>冲突</b>{{ c.path }} · {{ c.original || '' }} {{ c.reason }}</div>
          <div v-for="(w, i) in ir.warnings" :key="'w' + i" class="cw-issue warn"><b>提示</b>{{ w.path }} · {{ w.original }}(预期 {{ w.expected }}/实际 {{ w.actual }})已替换全部合法位置</div>
          <div v-for="(e, i) in ir.validationErrors" :key="'e' + i" class="cw-issue conflict"><b>渲染失败</b>{{ e.path }} · {{ e.error }}</div>

          <div class="cw-timeline" ref="logEl">
            <div v-if="activity.length === 0" class="cw-tl-empty">尚无过程记录——重新分析或调整文件/变量后,管线运行轨迹会实时出现在这里</div>
            <div v-for="(a, i) in activity" :key="i" class="cw-act">
              <span class="cw-act-dot" :class="a.stage"></span>
              <span class="cw-act-stage">{{ stageNames[a.stage] || a.stage }}</span>
              <span class="cw-act-text">{{ a.text }}</span>
              <span class="cw-act-ts">{{ fmtTs(a.ts) }}</span>
            </div>
          </div>
        
        </div>
      </div>

      <!-- 转换助手(Agent):本地工具+bash+IR 操作,最右栏;见设计文档 §12 -->
      <ConvertAgentPanel
        v-if="aiDock"
        :snapshot="agentSnapshot"
        :exec-op="execAgentOp"
        @close="aiDock = false"
        @log="onAgentLog"
      />
    </div>

    <!-- 构建验证结果 -->
    <a-modal :open="!!buildResult" title="构建验证" :footer="null" :width="640" @cancel="buildResult = null">
      <div v-if="buildResult" class="cw-bc">
        <div class="cw-bc-state" :class="buildResult.ok ? 'ok' : 'fail'">
          {{ buildResult.ok ? '✔ 构建通过' : (buildResult.stage === 'render' ? '✘ 渲染失败' : '✘ 构建失败') }}
          <span v-if="buildResult.durationMs != null" class="cw-bc-dur">{{ buildResult.durationMs }}ms</span>
        </div>
        <div v-if="buildResult.stage === 'render'" class="cw-bc-row">文件:{{ buildResult.path }}</div>
        <div v-if="buildResult.command" class="cw-bc-row cw-mono">$ {{ buildResult.command }}<span v-if="buildResult.exitCode != null && !buildResult.ok">(exit {{ buildResult.exitCode }})</span></div>
        <div v-if="buildResult.error" class="cw-bc-err">{{ buildResult.error }}</div>
        <pre v-if="buildResult.output" class="cw-bc-log">{{ buildResult.output }}</pre>
        <div class="cw-bc-tip">产物目录:{{ buildResult.dir }}(临时,可随时清理;构建命令可在规则包 buildCmd 调整)</div>
      </div>
    </a-modal>

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
import { ref, reactive, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'
import {
  ArrowLeftOutlined, CloseOutlined, RedoOutlined, SaveOutlined, MinusOutlined, BorderOutlined,
  FileOutlined, FolderFilled, FolderOpenFilled, CopyOutlined, CloseCircleOutlined, BuildOutlined, FileTextFilled, StarFilled, ClockCircleFilled,
} from '@ant-design/icons-vue'
import ConvertAgentPanel from './components/ConvertAgentPanel.vue'
import CodeViewer from '@/components/common/CodeViewer.vue'
import VariableIcon from '@/components/icons/VariableIcon.vue'
import AiIcon from '@/components/icons/AiIcon.vue'
import { tauriApi } from '@/utils/tauriApi'
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
const lastSrc = ref('')
const tplType = ref('scaffold')          // scaffold | data_driven(创建入口选择,存模板时使用)
const focusChecked = ref([])             // 数据驱动重点文件勾选(a-tree keys,含目录级联)
const exposeAll = ref(true)              // 其它文件是否暴露给 AI(true=全部内容,false=聚焦勾选+目录结构)
const focusDirty = ref(false)            // 勾选在分析后有变更,提示重新分析
const isDataDriven = computed(() => tplType.value === 'data_driven')
const busy = ref(false)
const storing = ref(false)
const errorMsg = ref('')
const degraded = ref(false)
const storeOpen = ref(false)
const storeForm = reactive({ name: '', description: '' })
const draftId = ref('')
const stageState = ref({ clone: '', scan: '', analyze: '' })

// ---- 过程记录(convert://log 事件 + 前端本地事件) ----
const activity = ref([])
const stageNames = { clone: '克隆', scan: '扫描', analyze: '分析', apply: '替换', store: '存储', agent: '助手', build: '构建' }
const pushActivity = (stage, text) => activity.value.push({ stage, text, ts: Date.now() })
const cloneTail = computed(() => activity.value.slice(-12))
const logEl = ref(null)
const runLogEl = ref(null)
watch(() => activity.value.length, async () => {
  await nextTick()
  for (const el of [logEl.value, runLogEl.value]) if (el) el.scrollTop = el.scrollHeight
})

// ---- 中栏:文件预览 / 过程 ----
const activeView = ref('source')   // source | template | focus(Activity Bar)
const bottomOpen = ref(true)         // 底部过程面板
const selectedPath = ref('')
const previewMode = ref('tpl')
const previewRaw = ref('')
const previewError = ref('')
const renderedContent = ref('')
const renderError = ref('')

const aiDock = ref(localStorage.getItem('convert-ai-dock') === '1')
watch(aiDock, (v) => localStorage.setItem('convert-ai-dock', v ? '1' : '0'))

// ---- 转换助手(Agent)宿主适配 ----
const agentSnapshot = computed(() => ({
  dir: ir.source.dir,
  source: ir.source.source,
  packId: ir.source.packId,
  tplType: tplType.value,
  files: ir.files.map(({ path, action }) => ({ path, action })),
  variables: ir.variables.map(({ name, defaultValue, enabled }) => ({ name, defaultValue, enabled })),
  focusFiles: focusFiles.value,
  exposeAll: exposeAll.value,
  logTail: activity.value.slice(-20),
}))
const execAgentOp = (name, args) => {
  if (name === 'set_file_action') {
    const f = ir.files.find((x) => x.path === args.path)
    if (!f) return `错误:文件 ${args.path} 不在清单中(新文件需重新扫描)`
    f.action = args.action === 'exclude' ? 'exclude' : 'keep'
    f.reason = f.action === 'exclude' ? '助手剔除' : ''
    return `已${f.action === 'keep' ? '保留' : '剔除'} ${args.path}`
  }
  if (name === 'set_focus') {
    const set = new Set((args.files || []).map(String))
    focusChecked.value = [...set]
    if (typeof args.expose_all === 'boolean') exposeAll.value = args.expose_all
    return `重点文件已设为 ${set.size} 个(暴露策略:${exposeAll.value ? '全部' : '聚焦勾选'})`
  }
  if (name === 'update_variable') {
    const v = ir.variables.find((x) => x.name === args.name)
    if (!v) return `错误:变量 ${args.name} 不存在`
    if (args.new_name) v.name = String(args.new_name)
    if (args.default_value !== undefined) v.defaultValue = String(args.default_value)
    if (typeof args.enabled === 'boolean') v.enabled = args.enabled
    return `变量 ${args.name} 已更新`
  }
  if (name === 'rerun_analysis') {
    if (busy.value) return '分析进行中,请稍候'
    rerunAnalyze()
    return '已触发重新分析,结果见「转换过程」'
  }
  return `未知操作 ${name}`
}
const onAgentLog = (text) => pushActivity('agent', text)

const tid = () => draftId.value
const keptCount = computed(() => ir.files.filter((f) => f.action === 'keep').length)
const filePathSet = computed(() => new Set(ir.files.map((f) => f.path)))
const focusFiles = computed(() => focusChecked.value.filter((k) => filePathSet.value.has(k)))
const enabledVars = computed(() => ir.variables.filter((v) => v.enabled))
const canStore = computed(() => ir.source.dir && keptCount.value > 0 && !busy.value)
const pipelineRunning = computed(() => Object.values(stageState.value).some((s) => s === 'run'))
const srcShort = computed(() => {
  const s = ir.source.source || ''
  return s.length > 42 ? s.slice(0, 18) + '…' + s.slice(-20) : s
})

// 管线运行时自动切到过程页
watch(pipelineRunning, (r) => { if (r && ir.source.dir) bottomOpen.value = true })

const stages = computed(() => [
  { label: '克隆', state: stageState.value.clone, detail: ir.source.commit ? ir.source.commit.slice(0, 7) : '' },
  { label: '扫描', state: stageState.value.scan, detail: ir.files.length ? `${keptCount.value}/${ir.files.length} 保留` : '' },
  { label: '分析', state: stageState.value.analyze, detail: ir.variables.length ? `${ir.variables.length} 候选变量` : '' },
  { label: '存储', state: '', detail: '' },
])

// ---- 文件树(嵌套) ----
const expandedKeys = ref([])
const expandedSet = computed(() => new Set(expandedKeys.value.map(String)))
const buildTreeFiles = (files) => {
  const dirMap = new Map()
  const roots = []
  const ensureDir = (dirPath) => {
    if (!dirPath) return roots
    if (dirMap.has(dirPath)) return dirMap.get(dirPath).children
    const parts = dirPath.split('/')
    const name = parts.pop()
    const parent = ensureDir(parts.join('/'))
    const node = { key: dirPath, title: name, isDir: true, children: [] }
    dirMap.set(dirPath, node)
    parent.push(node)
    return node.children
  }
  for (const f of files) ensureDir(f.dir).push({ key: f.path, title: f.base, isDir: false, file: f })
  const sortNodes = (arr) => {
    arr.sort((a, b) => (a.isDir === b.isDir ? a.title.localeCompare(b.title) : a.isDir ? -1 : 1))
    arr.forEach((n) => n.isDir && sortNodes(n.children))
  }
  sortNodes(roots)
  return roots
}
const treeData = computed(() => buildTreeFiles(ir.files))
const tplTreeData = computed(() => buildTreeFiles(keptFiles.value))
// 默认全折叠(VSCode 式),用户自行展开
const expandRoots = () => { expandedKeys.value = [] }
const filesUnder = (dir) => ir.files.filter((f) => f.path.startsWith(dir + '/'))
const dirKept = (dir) => filesUnder(dir).filter((f) => f.action === 'keep').length
const dirAllExcluded = (dir) => { const s = filesUnder(dir); return s.length > 0 && s.every((f) => f.action === 'exclude') }
const toggleFile = (f) => {
  f.action = f.action === 'keep' ? 'exclude' : 'keep'
  f.reason = f.action === 'exclude' ? '手动剔除' : ''
}
const toggleDir = (dir) => {
  const set = filesUnder(dir)
  const toKeep = dirAllExcluded(dir)
  for (const f of set) { f.action = toKeep ? 'keep' : 'exclude'; f.reason = toKeep ? '' : '手动剔除' }
}
const onTreeExpand = (keys) => { expandedKeys.value = keys }
const onTreeSelect = (keys, info) => {
  const d = info?.node?.dataRef || info?.node || {}
  if (!d.isDir && d.file) {
    selectedPath.value = String(d.key)
    previewMode.value = 'raw'
  }
}
const closePreview = () => { selectedPath.value = '' }

// 勾选/暴露策略变更:落草稿 + 若分析已完成则提示重新分析生效
watch([() => focusChecked.value.join('|'), exposeAll], () => {
  scheduleSave()
  if (stageState.value.analyze === 'done') focusDirty.value = true
})

// ---- 预览(行号 + 占位符高亮;原文走 convert_read_file,模板化走 apply 结果) ----
const tplOutput = computed(() => ir.outputs.find((o) => o.path === selectedPath.value) || null)
const previewContent = computed(() => {
  if (previewMode.value === 'raw') return previewRaw.value
  if (previewMode.value === 'render') return renderedContent.value
  return tplOutput.value ? tplOutput.value.content : previewRaw.value
})
const previewReplaced = computed(() => (previewMode.value === 'tpl' && tplOutput.value ? tplOutput.value.replaced || 0 : 0))
const tplStale = computed(() => previewMode.value === 'tpl' && selectedPath.value && !tplOutput.value && ir.variables.length > 0)

const escHighlight = (line) => {
  const esc = String(line).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  return esc.replace(/\{\{[^{}]{1,160}\}\}/g, (m) => `<span class="cw-ph">${m}</span>`)
}
const MAX_LINES = 3000
const previewLines = computed(() => {
  const lines = String(previewContent.value || '').split('\n')
  const out = lines.slice(0, MAX_LINES).map(escHighlight)
  if (lines.length > MAX_LINES) out.push(`<span class="dim">… 已截断(共 ${lines.length} 行)</span>`)
  return out.length ? out : ['']
})

watch(selectedPath, async (p) => {
  previewRaw.value = ''
  previewError.value = ''
  renderedContent.value = ''
  renderError.value = ''
  if (!p) return
  try {
    const raw = await invoke('convert_read_file', { root: ir.source.dir, path: p })
    previewRaw.value = JSON.parse(raw).content ?? ''
  } catch (e) {
    previewError.value = String(e)
  }
})
const regenPreview = async () => {
  if (busy.value) return
  try { await runApply() } catch (e) { message.error('生成失败: ' + (e.message || e)) }
}

// ---- 模板文件页(模板区) ----
const keptFiles = computed(() => ir.files.filter((f) => f.action === 'keep'))
const onTplTreeSelect = (keys, info) => {
  const d = info?.node?.dataRef || info?.node || {}
  if (!d.isDir && d.file) onTplSelect(String(d.key))
}
const onTplSelect = async (path) => {
  selectedPath.value = path
  previewMode.value = 'tpl'
  if (!ir.outputs.length && !busy.value) {
    try { await runApply() } catch (e) { message.error('生成替换结果失败: ' + (e.message || e)) }
  }
}
const activeLabel = computed(() => ({ source: '源代码区', template: '模板区', focus: '重点文件', vars: '变量' }[activeView.value] || ''))
const emptyHint = computed(() => (activeView.value === 'vars'
  ? '尚无候选变量——完成分析后在「变量」中调整'
  : activeView.value === 'template'
  ? '在左侧选择模板文件,查看模板化内容;绿色高亮为变量占位符'
  : '点击左侧文件查看内容;行尾「保留/剔除」控制是否进入模板'))
const tplReplacedOf = (path) => ir.outputs.find((o) => o.path === path)?.replaced || 0
const buildChecking = ref(false)
const buildResult = ref(null)
const runBuildCheck = async () => {
  if (buildChecking.value) return
  buildChecking.value = true
  buildResult.value = null
  try {
    if (!ir.outputs.length) await runApply()
    const vars = {}
    for (const v of enabledVars.value) { const n = (v.name || '').trim(); if (n) vars[n] = v.defaultValue ?? '' }
    pushActivity('build', '存储前构建验证:渲染落盘 + ' + (ir.source.packId || '') + ' 构建冒烟…')
    const raw = await invoke('convert_build_check', { outputs: ir.outputs, variables: vars, packId: ir.source.packId })
    buildResult.value = JSON.parse(raw)
  } catch (e) {
    buildResult.value = { ok: false, stage: 'error', error: String(e) }
  } finally {
    buildChecking.value = false
  }
}

// ---- 整体渲染预览页 ----
const raLoading = ref(false)
const doRenderPreview = async () => {
  renderError.value = ''
  if (!selectedPath.value) { renderedContent.value = ''; return }
  try {
    if (!ir.outputs.length) await runApply()
    const o = ir.outputs.find((x) => x.path === selectedPath.value)
    if (!o) { renderedContent.value = ''; return }
    const vars = {}
    for (const v of enabledVars.value) { const n = (v.name || '').trim(); if (n) vars[n] = v.defaultValue ?? '' }
    const r = await invoke('render_string_content', { template: o.content, variables: vars })
    if (r && r.success) renderedContent.value = r.content || ''
    else { renderedContent.value = ''; renderError.value = r?.error?.message || r?.error?.type || '未知错误' }
  } catch (e) {
    renderedContent.value = ''
    renderError.value = String(e)
  }
}
watch([previewMode, selectedPath, () => enabledVars.value.map((v) => [v.name, v.defaultValue]).join('|')], () => {
  if (previewMode.value === 'render') doRenderPreview()
  else { renderedContent.value = ''; renderError.value = '' }
})

// ---- 管线 ----
const startConvert = async (src, branch = null) => {
  if (busy.value || !src) return
  busy.value = true
  errorMsg.value = ''
  lastSrc.value = src
  activity.value = []
  try {
    stageState.value = { clone: 'run', scan: '', analyze: '' }
    const raw = await invoke('convert_clone', { source: src, branch })
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
    dir: f.path.includes('/') ? f.path.slice(0, f.path.lastIndexOf('/')) : '',
    action: f.action === 'exclude' ? 'exclude' : 'keep',
  }))
  expandRoots()
  stageState.value.scan = 'done'
  // 数据驱动:扫描后停在「重点标注」步骤(可选),由用户手动开始分析;脚手架:自动串行
  if (isDataDriven.value) {
    stageState.value.analyze = 'wait'
    activeView.value = 'focus'
    pushActivity('analyze', '数据驱动模式:在「重点文件」中勾选关键文件并选择 AI 暴露范围,点「下一步」开始分析(也可跳过)')
  } else {
    await doAnalyze()
  }
}

const doAnalyze = async () => {
  stageState.value.analyze = 'run'
  const keeps = ir.files.filter((f) => f.action === 'keep').map((f) => f.path)
  try {
    const raw = await invoke('convert_analyze', {
      root: ir.source.dir, files: keeps, provider: null, model: null, thinking: null,
      focusFiles: isDataDriven.value ? focusFiles.value : null,
      exposeAllFiles: isDataDriven.value ? exposeAll.value : null,
    })
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
    focusDirty.value = false
  } catch (e) {
    degraded.value = true
    stageState.value.analyze = 'error'
    message.error('分析失败: ' + (e.message || e))
  }
}
const rerunAnalyze = () => { if (!busy.value) { busy.value = true; doAnalyze().finally(() => { busy.value = false }) } }

// ---- 替换(确定性) ----
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

// 状态变化使替换结果失效
watch(() => ir.files.map((f) => f.path + f.action).join('|') + JSON.stringify(enabledVars.value.map((v) => [v.name, v.defaultValue])), () => {
  ir.outputs = []
  renderedContent.value = ''
  renderError.value = ''
  ir.conflicts = []; ir.warnings = []; ir.validationErrors = []
})

// ---- 存储 ----
const openStore = async () => {
  busy.value = true
  try {
    pushActivity('store', '校验替换结果…')
    const r = await runApply()
    if (!r.clean) {
      message.warning('存在冲突或渲染失败,已在「转换过程」页列出,请处理后重试')
      bottomOpen.value = true
      return
    }
    if (!storeForm.name) storeForm.name = ir.source.source.split(/[\\/]/).pop()?.replace(/\.git$/, '') || '转换模板'
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
    pushActivity('store', `创建模板「${storeForm.name.trim()}」…`)
    const outputs = ir.outputs.length ? ir.outputs : (await runApply()).outputs
    const res = await createUserTemplate({
      name: storeForm.name.trim(), templateType: tplType.value, categoryId: null,
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
    pushActivity('store', `模板创建成功(${outputs.length} 文件),已进入编辑器`)
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
        meta: { source: ir.source, degraded: degraded.value, tplType: tplType.value, savedAt: Date.now() },
        ir: {
          files: ir.files.map(({ path, action, reason }) => ({ path, action, reason })),
          variables: ir.variables,
          focus: isDataDriven.value ? { files: focusFiles.value, exposeAll: exposeAll.value } : null,
        },
      })
    } catch { /* 静默 */ }
  }, 800)
}
watch([() => ir.files.map((f) => f.path + f.action).join('|'), () => JSON.stringify(ir.variables)], scheduleSave)

const openDraft = async (id) => {
  try {
    const raw = await invoke('convert_draft_load', { id })
    const d = JSON.parse(raw)
    if (!d.meta?.source?.dir) { message.error('草稿损坏'); return }
    draftId.value = id
    Object.assign(ir.source, d.meta.source)
    degraded.value = !!d.meta.degraded
    tplType.value = d.meta?.tplType === 'data_driven' ? 'data_driven' : 'scaffold'
    ir.files = (d.ir?.files || []).map((f) => ({ ...f, base: f.path.split('/').pop(), dir: f.path.includes('/') ? f.path.slice(0, f.path.lastIndexOf('/')) : '' }))
    ir.variables = d.ir?.variables || []
    const f = d.ir?.focus
    focusChecked.value = (f?.files || []).map(String)
    exposeAll.value = f ? f.exposeAll !== false : true
    expandRoots()
    stageState.value = { clone: 'done', scan: 'done', analyze: 'done' }
    pushActivity('clone', `恢复草稿:${ir.source.source}`)
  } catch (e) { message.error('打开草稿失败: ' + (e.message || e)) }
}

// ---- 杂项 ----
const fmtTs = (ts) => {
  const d = new Date(Number(ts))
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}:${String(d.getSeconds()).padStart(2, '0')}`
}
const confColor = (c) => (c >= 0.7 ? 'var(--color-brand, #16a34a)' : c >= 0.5 ? '#d97706' : '#999')
const occTooltip = (v) => (v.occurrences || []).map((o) => `${o.path} ×${o.count}`).join('\n')
const winMin = async () => { try { await tauriApi.window.minimize() } catch {} }
const winMax = async () => { try { await tauriApi.window.maximize() } catch {} }
const winClose = async () => { try { await tauriApi.window.close() } catch {} }

// ---- 事件监听(convert://log 过程流) ----
let unlistenLog = null
onMounted(async () => {
  // 参数消费:src/branch/type(引导页或新建弹窗发起)→ 自动开跑;draft → 恢复;空参数 → 回落引导页
  if (route.query.draft) {
    openDraft(String(route.query.draft))
  } else if (route.query.src) {
    const src = String(route.query.src)
    const branch = route.query.branch ? String(route.query.branch) : null
    tplType.value = route.query.type === 'data_driven' ? 'data_driven' : 'scaffold'
    router.replace({ query: {} })
    startConvert(src, branch)
  } else if (!ir.source.dir) {
    router.replace('/convert')
  }
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlistenLog = await listen('convert://log', (e) => {
      const { stage, text } = e.payload || {}
      if (stage && text) pushActivity(stage, text)
    })
  } catch { /* 非 tauri 环境(浏览器 dev)无事件 */ }
})
onBeforeUnmount(() => { unlistenLog?.() })
</script>

<style scoped>
.cw-page { height: 100%; display: flex; flex-direction: column; background: var(--editor-bg, #f5f5f5); }

/* ===== 顶栏 ===== */
.cw-head { height: 48px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; padding: 0 8px 0 12px; background: var(--editor-panel-bg, #fff); border-bottom: 1px solid var(--editor-border, #e2e8f0); }
.cw-head-left { display: flex; align-items: center; gap: 8px; min-width: 0; }
.cw-title { font-size: 13px; font-weight: 600; color: var(--color-text, #1b1c1f); letter-spacing: 0.3px; }
.cw-vbar { width: 1px; height: 16px; background: var(--editor-border, #e2e8f0); }
.cw-src { max-width: 320px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; color: var(--color-text-secondary, #666); }
.cw-mono { font-family: Consolas, 'JetBrains Mono', monospace; }
.cw-head-right { display: flex; align-items: center; gap: 8px; }
.cw-win { display: flex; align-items: center; gap: 2px; margin-left: 6px; }
.cw-icon-btn { display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; border: none; background: transparent; border-radius: 6px; cursor: pointer; color: var(--color-text-secondary, #64748b); font-size: 14px; transition: background 0.15s, color 0.15s; }
.cw-icon-btn:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.cw-icon-btn.danger:hover { background: #fee2e2; color: #dc2626; }
.cw-icon-btn.sm { width: 22px; height: 22px; font-size: 12px; }

/* ===== 入口屏 ===== */
.cw-source { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 24px; padding: 24px; overflow-y: auto; }
.cw-source-card { width: min(640px, 100%); background: var(--editor-panel-bg, #fff); border-radius: 12px; padding: 24px; box-shadow: var(--shadow-panel, 0 1px 3px rgba(0,0,0,0.06)); display: flex; flex-direction: column; gap: 12px; }
.cw-source-title { font-size: 16px; font-weight: 600; color: var(--color-text, #1b1c1f); }
.cw-source-sub { font-size: 12px; color: var(--color-text-secondary, #999); }
.cw-src-now { font-size: 12px; color: var(--color-text-secondary, #666); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-run-stages { display: flex; gap: 18px; }
.cw-rstage { display: flex; align-items: center; gap: 6px; font-size: 12.5px; color: var(--color-text-secondary, #999); }
.cw-rdot { width: 8px; height: 8px; border-radius: 50%; background: var(--editor-border, #ddd); }
.cw-rstage.done .cw-rdot { background: var(--color-brand, #16a34a); }
.cw-rstage.run .cw-rdot { background: #d97706; animation: cw-breathe 1.2s ease-in-out infinite; }
.cw-rstage.error .cw-rdot { background: #dc2626; }
.cw-run-log { background: var(--color-canvas, #f6f8fa); border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; padding: 10px 12px; max-height: 220px; min-height: 120px; overflow-y: auto; font-family: Consolas, 'JetBrains Mono', monospace; font-size: 11.5px; line-height: 1.7; color: var(--color-text-secondary, #555); }
.cw-run-line { white-space: pre-wrap; word-break: break-all; }
.cw-run-ts { color: var(--color-text-muted, #b0b0aa); margin-right: 8px; }
.cw-run-line.dim { color: var(--color-text-muted, #aaa); }
@keyframes cw-breathe { 0%, 100% { opacity: 0.35; } 50% { opacity: 1; } }
.cw-source-row { display: flex; gap: 8px; align-items: center; }
.cw-error { color: #dc2626; font-size: 12px; white-space: pre-wrap; }

/* ===== 三栏骨架 ===== */
.cw-main { flex: 1; min-height: 0; display: flex; }
/* Activity Bar(VSCode 式) */
.cw-actbar { width: 44px; flex-shrink: 0; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 8px 0; background: var(--editor-panel-bg, #fff); border-right: 1px solid var(--editor-border, #e2e8f0); }
.cw-act-item { position: relative; width: 36px; height: 36px; border: none; background: transparent; border-radius: 8px; cursor: pointer; color: var(--color-text-secondary, #64748b); display: flex; align-items: center; justify-content: center; }
.cw-act-item:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.cw-act-item.on { background: var(--color-nav-active, #eef0ec); color: var(--color-text, #1b1c1f); }
.cw-act-item.on::before { content: ''; position: absolute; left: -4px; top: 8px; bottom: 8px; width: 2.5px; border-radius: 2px; background: var(--color-brand, #16a34a); }
.cw-act-ico { font-size: 19px; }
.cw-act-n { position: absolute; right: 0; top: 0; min-width: 13px; height: 13px; padding: 0 3px; border-radius: 7px; background: #3e7bfa; color: #fff; font-size: 9px; display: flex; align-items: center; justify-content: center; }
.cw-act-live { position: absolute; right: 4px; top: 4px; width: 6px; height: 6px; border-radius: 50%; background: #d97706; animation: cw-breathe 1.2s ease-in-out infinite; }
/* Side Bar */
.cw-sidebar { width: 250px; flex-shrink: 0; display: flex; flex-direction: column; min-height: 0; background: var(--editor-panel-bg, #fff); border-right: 1px solid var(--editor-border, #e2e8f0); }
.cw-ra-list.flat { width: auto; flex: 1; min-height: 0; border-right: none; padding: 6px 8px 12px; }
/* 中栏列(内容 + 底部过程) */
.cw-center-col { flex: 1; min-width: 0; display: flex; flex-direction: column; min-height: 0; }
.cw-chead { height: 40px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 0 12px 0 14px; border-bottom: 1px solid var(--editor-border, #e2e8f0); background: var(--editor-panel-bg, #fff); }
.cw-crumb { font-size: 12px; color: var(--color-text-secondary, #555); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-chead-right { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
/* 底部过程面板 */
.cw-bottom { height: 230px; flex-shrink: 0; display: flex; flex-direction: column; border-top: 1px solid var(--editor-border, #e2e8f0); background: var(--editor-bg, #fafbfc); min-height: 0; }
.cw-bottom-head { height: 32px; flex-shrink: 0; display: flex; align-items: center; gap: 10px; padding: 0 8px 0 14px; font-size: 12px; font-weight: 600; color: var(--color-text-secondary, #64748b); border-bottom: 1px solid var(--editor-border, #e2e8f0); background: var(--editor-panel-bg, #fff); }
.cw-bottom-head .cw-icon-btn { margin-left: auto; }
.cw-panel { background: var(--editor-panel-bg, #fff); display: flex; flex-direction: column; min-height: 0; flex-shrink: 0; }
.cw-left { width: 280px; border-right: 1px solid var(--editor-border, #e2e8f0); }
.cw-right { width: 320px; border-left: 1px solid var(--editor-border, #e2e8f0); }
.cw-panel-head { height: 40px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; padding: 0 12px; font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; color: var(--color-text-secondary, #64748b); border-bottom: 1px solid var(--editor-border, #e2e8f0); }
.cw-count { font-weight: 400; text-transform: none; letter-spacing: 0; color: var(--color-text-muted, #94a3b8); }

/* ===== 文件树 ===== */
.cw-tree { flex: 1; min-height: 0; overflow-y: auto; padding: 6px 8px 12px; }
.cw-tree::-webkit-scrollbar, .cw-vars::-webkit-scrollbar, .cw-code::-webkit-scrollbar, .cw-timeline::-webkit-scrollbar, .cw-run-log::-webkit-scrollbar { width: 6px; }
.cw-tree::-webkit-scrollbar-thumb, .cw-vars::-webkit-scrollbar-thumb, .cw-code::-webkit-scrollbar-thumb, .cw-timeline::-webkit-scrollbar-thumb, .cw-run-log::-webkit-scrollbar-thumb { background: #c8c8c4; border-radius: 3px; }
.cw-fic { font-size: 13px; color: var(--color-text-muted, #8a919c); }
.cw-fic.dir { color: #eab308; }
.cw-row { display: flex; align-items: center; gap: 6px; min-width: 0; font-size: 12.5px; line-height: 1; padding: 4px 2px; }
.cw-row.excluded .cw-name { color: var(--color-text-muted, #9aa0a6); text-decoration: line-through; }
.cw-name { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; background: var(--color-brand, #16a34a); }
.cw-dot.exclude { background: #dc2626; }
.cw-entry { flex: none; font-size: 9px; color: var(--color-brand, #16a34a); border: 1px solid currentColor; border-radius: 4px; padding: 1px 3px 0; }
.cw-dircount { flex: none; font-size: 10px; color: var(--color-text-muted, #9aa0a6); }
.cw-toggle { flex: none; visibility: hidden; border: 1px solid var(--editor-border, #d8dde3); background: transparent; color: var(--color-brand, #16a34a); font-size: 10px; line-height: 1; padding: 3px 6px; border-radius: 4px; cursor: pointer; }
.cw-toggle.off { color: #dc2626; }
.cw-row:hover .cw-toggle, .cw-toggle:focus-visible { visibility: visible; }
.cw-toggle:hover { border-color: currentColor; }
:deep(.ant-tree-treenode) { padding: 0; }
:deep(.ant-tree-node-content-wrapper) { display: inline-flex; align-items: center; min-width: 0; flex: 1; }
:deep(.ant-tree .ant-tree-node-content-wrapper:hover) { background: transparent; }

/* ===== 中栏 ===== */
.cw-center { flex: 1; min-width: 0; display: flex; flex-direction: column; background: var(--editor-bg, #fafbfc); }
.cw-tabs { height: 40px; flex-shrink: 0; display: flex; align-items: center; gap: 2px; padding: 0 12px; border-bottom: 1px solid var(--editor-border, #e2e8f0); background: var(--editor-panel-bg, #fff); }
.cw-tab { position: relative; border: none; background: transparent; font-size: 12.5px; color: var(--color-text-secondary, #64748b); padding: 6px 10px; border-radius: 6px; cursor: pointer; }
.cw-tab:hover:not(:disabled) { background: var(--color-hover, #f1f5f9); }
.cw-tab.active { color: var(--color-text, #1b1c1f); font-weight: 600; background: var(--color-nav-active, #eef0ec); }
.cw-tab:disabled { opacity: 0.45; cursor: not-allowed; }
.cw-live { display: inline-block; width: 6px; height: 6px; border-radius: 50%; background: #d97706; margin-left: 6px; animation: cw-breathe 1.2s ease-in-out infinite; }
.cw-tabs-right { margin-left: auto; display: flex; align-items: center; gap: 10px; }
/* 分区:源代码区/模板区(zone 标签 + 分隔) */
.cw-zone { font-size: 10px; font-weight: 600; letter-spacing: 0.5px; color: var(--color-text-muted, #9aa0a6); padding: 2px 6px 2px 2px; user-select: none; white-space: nowrap; }
.cw-zone.tpl { color: var(--color-brand, #16a34a); }
.cw-zone-sep { width: 1px; height: 14px; background: var(--editor-border, #d8dde3); margin: 0 6px 0 2px; }
.cw-tab-n.dim2 { background: var(--color-border, #d8dde3); color: var(--color-text-secondary, #64748b); }
.cw-act-dot.build { background: #0ea5e9; }
.cw-bc { display: flex; flex-direction: column; gap: 10px; padding: 4px 0; }
.cw-bc-state { font-size: 14px; font-weight: 600; }
.cw-bc-state.ok { color: var(--color-brand, #16a34a); }
.cw-bc-state.fail { color: #dc2626; }
.cw-bc-dur { font-size: 11px; color: var(--color-text-muted, #9aa0a6); font-family: Consolas, monospace; margin-left: 8px; }
.cw-bc-row { font-size: 12.5px; color: var(--color-text, #333); }
.cw-bc-err { font-size: 12.5px; color: #b91c1c; background: rgba(220, 38, 38, 0.06); border-radius: 6px; padding: 8px 10px; word-break: break-all; }
.cw-bc-log { margin: 0; padding: 10px 12px; background: #1e1e2e; color: #cdd6f4; border-radius: 8px; font-family: Consolas, 'JetBrains Mono', monospace; font-size: 11.5px; line-height: 1.6; max-height: 320px; overflow: auto; white-space: pre-wrap; word-break: break-all; }
.cw-bc-tip { font-size: 11px; color: var(--color-text-muted, #9aa0a6); }
.cw-seg { display: flex; border: 1px solid var(--editor-border, #d8dde3); border-radius: 6px; overflow: hidden; }
.cw-seg button { border: none; background: transparent; font-size: 11.5px; padding: 3px 10px; cursor: pointer; color: var(--color-text-secondary, #64748b); }
.cw-seg button.on { background: var(--color-nav-active, #eef0ec); color: var(--color-text, #1b1c1f); font-weight: 600; }
.cw-repl { font-size: 11px; color: var(--color-brand, #16a34a); }

/* 过程页 */
.cw-proc { flex: 1; min-height: 0; display: flex; flex-direction: column; padding: 8px 14px 10px; gap: 8px; overflow: hidden; }
.cw-stagestrip { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; flex-shrink: 0; }
.cw-pstage { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--color-text-secondary, #64748b); }
.cw-pdot { width: 9px; height: 9px; border-radius: 50%; background: var(--editor-border, #d5dbe1); }
.cw-pstage.done { color: var(--color-text, #1b1c1f); }
.cw-pstage.done .cw-pdot { background: var(--color-brand, #16a34a); }
.cw-pstage.run .cw-pdot { background: #d97706; animation: cw-breathe 1.2s ease-in-out infinite; }
.cw-pstage.error .cw-pdot { background: #dc2626; }
.cw-pdetail { font-size: 11px; color: var(--color-text-muted, #9aa0a6); }
.cw-plink { width: 26px; height: 1px; background: var(--editor-border, #d5dbe1); }
.cw-issue { font-size: 12px; padding: 6px 10px; border-radius: 8px; flex-shrink: 0; }
.cw-issue b { margin-right: 6px; }
.cw-issue.conflict { background: rgba(220, 38, 38, 0.06); color: #b91c1c; }
.cw-issue.warn { background: rgba(217, 119, 6, 0.06); color: #b45309; }
.cw-timeline { flex: 1; min-height: 0; overflow-y: auto; background: var(--editor-panel-bg, #fff); border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; padding: 8px 0; }
.cw-tl-empty { padding: 18px 16px; font-size: 12px; color: var(--color-text-muted, #9aa0a6); }
.cw-act { display: flex; align-items: baseline; gap: 8px; padding: 3px 12px; font-size: 12px; }
.cw-act:hover { background: var(--color-surface, #f7f7f5); }
.cw-act-dot { width: 6px; height: 6px; border-radius: 50%; flex: none; align-self: center; background: var(--color-text-muted, #9aa0a6); }
.cw-act-dot.clone { background: #3e7bfa; }
.cw-act-dot.scan { background: #8b5cf6; }
.cw-act-dot.analyze { background: #d97706; }
.cw-act-dot.apply { background: #0ea5e9; }
.cw-act-dot.store { background: var(--color-brand, #16a34a); }
.cw-act-dot.agent { background: #8b5cf6; }
.cw-act-stage { flex: none; font-size: 10.5px; color: var(--color-text-muted, #9aa0a6); width: 26px; }
.cw-act-text { flex: 1; min-width: 0; color: var(--color-text, #333); word-break: break-all; }
.cw-act-ts { flex: none; font-size: 10px; color: var(--color-text-muted, #b6bcc2); font-family: Consolas, monospace; }

/* 文件预览页 */
.cw-filepane { flex: 1; min-height: 0; display: flex; flex-direction: column; }
.cw-empty { flex: 1; display: flex; align-items: center; justify-content: center; font-size: 12.5px; color: var(--color-text-muted, #9aa0a6); padding: 24px; text-align: center; }
.cw-file-head { height: 34px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; padding: 0 8px 0 14px; background: var(--editor-panel-bg, #fff); border-bottom: 1px solid var(--editor-border, #e2e8f0); font-size: 12px; color: var(--color-text-secondary, #555); }
.cw-stale { display: flex; align-items: center; gap: 8px; padding: 6px 14px; font-size: 12px; color: #b45309; background: rgba(217, 119, 6, 0.06); flex-shrink: 0; }
.cw-code { flex: 1; min-height: 0; overflow: auto; background: var(--editor-panel-bg, #fff); padding: 8px 0; font-family: Consolas, 'JetBrains Mono', monospace; font-size: 12px; line-height: 1.6; }
.cw-ln { display: flex; }
.cw-no { flex: none; width: 46px; text-align: right; padding-right: 12px; color: var(--color-text-muted, #b6bcc2); user-select: none; }
.cw-lc { white-space: pre; color: var(--color-text, #333); }
:deep(.cw-ph) { background: rgba(22, 163, 74, 0.12); color: var(--color-brand, #15803d); border-radius: 3px; padding: 0 1px; }
:deep(.dim) { color: var(--color-text-muted, #9aa0a6); }

/* ===== 变量面板 ===== */
.cw-vars { flex: 1; min-height: 0; overflow-y: auto; padding: 8px 10px 12px; }
.cw-var { border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; padding: 8px 10px; margin-bottom: 6px; display: flex; flex-direction: column; gap: 4px; }
.cw-var.off { opacity: 0.55; }
.cw-var-top { display: flex; align-items: center; gap: 6px; }
.cw-var-name { flex: 1; min-width: 0; border: none; outline: none; font-size: 12.5px; font-weight: 600; color: var(--color-text, #1b1c1f); background: transparent; }
.cw-var-name:focus { border-bottom: 1px solid var(--color-brand, #16a34a); }
.cw-conf { font-size: 10px; flex: none; }
.cw-var-mid { display: flex; align-items: center; gap: 8px; }
.cw-var-sem { flex: none; font-size: 10px; color: var(--color-text-secondary, #999); border: 1px solid var(--editor-border, #e2e8f0); border-radius: 4px; padding: 0 4px; }
.cw-var-def { flex: 1; min-width: 0; border: none; outline: none; font-size: 12px; color: var(--color-text-secondary, #666); background: transparent; font-family: Consolas, monospace; }
.cw-var-def:focus { border-bottom: 1px solid var(--color-brand, #16a34a); }
.cw-var-occ { font-size: 10.5px; color: var(--color-text-secondary, #999); }
.cw-store-sum { font-size: 12px; color: var(--color-text-secondary, #999); }

.cw-rapane { flex: 1; min-height: 0; display: flex; overflow: hidden; }
.cw-ra-list { width: 240px; flex-shrink: 0; border-right: 1px solid var(--editor-border, #e2e8f0); overflow-y: auto; background: var(--editor-panel-bg, #fff); padding: 6px 6px 12px; }
.cw-ra-item { display: flex; flex-direction: column; gap: 1px; padding: 6px 8px; border-radius: 6px; cursor: pointer; position: relative; }
.cw-ra-item:hover { background: var(--color-surface, #f7f7f5); }
.cw-ra-item.cur { background: var(--color-nav-active, #eef0ec); }
.cw-ra-name { font-size: 12px; color: var(--color-text, #333); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-ra-item.cur .cw-ra-name { font-weight: 600; }
.cw-ra-path { font-size: 10px; color: var(--color-text-muted, #9aa0a6); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; direction: rtl; text-align: left; }
.cw-ra-badge { position: absolute; right: 6px; top: 6px; min-width: 14px; height: 14px; padding: 0 3px; border-radius: 7px; background: var(--color-brand, #16a34a); color: #fff; font-size: 9px; display: flex; align-items: center; justify-content: center; }
.cw-ra-view { flex: 1; min-width: 0; display: flex; flex-direction: column; overflow: hidden; background: var(--editor-panel-bg, #fff); }
.cw-ra-head { height: 40px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 0 10px 0 14px; border-bottom: 1px solid var(--editor-border, #e2e8f0); }
.cw-ra-hpath { font-size: 12px; color: var(--color-text-secondary, #555); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cw-ra-meta { display: flex; align-items: center; gap: 10px; flex-shrink: 0; }
.engine-tag { font-size: 10px; font-weight: 600; color: #64748b; border: 1px solid #cbd5e1; border-radius: 4px; padding: 0 5px; letter-spacing: 0.3px; }
.render-time { font-size: 11px; color: var(--color-text-muted, #9aa0a6); font-family: Consolas, monospace; }
.cw-ra-code { flex: 1; min-height: 0; }
.cw-code-cm { flex: 1; min-height: 0; }
.cw-ra-error { flex: 1; min-height: 0; overflow-y: auto; margin: 14px; padding: 14px 16px; background: #fff5f4; border: 1px solid #ffd7d3; border-radius: 8px; display: flex; flex-direction: column; gap: 8px; }
.ra-err-head { display: flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; color: #d93025; }
.ra-err-type { font-size: 12px; color: #b3261e; font-weight: 600; }
.ra-err-msg { font-size: 12.5px; color: #5f2120; line-height: 1.6; word-break: break-all; }
.ra-err-ctx { margin: 0; padding: 8px 10px; background: rgba(255, 255, 255, 0.7); border-radius: 6px; font-family: Consolas, monospace; font-size: 11.5px; color: #5f2120; white-space: pre-wrap; }

/* ===== 数据驱动:重点文件勾选 ===== */
.cw-tab-n { display: inline-flex; align-items: center; justify-content: center; min-width: 15px; height: 15px; padding: 0 4px; margin-left: 5px; border-radius: 8px; background: #3e7bfa; color: #fff; font-size: 10px; }
.cw-focuspane { flex: 1; min-height: 0; display: flex; flex-direction: column; padding: 10px 12px; gap: 8px; overflow: hidden; }
/* 重点文件主内容区:左树右策略 */
.cw-focus-main { flex: 1; min-height: 0; display: flex; gap: 14px; padding: 14px 16px; }
.cw-focus-treebox { flex: 1.5; min-width: 0; display: flex; flex-direction: column; }
.cw-focus-side { width: 320px; flex-shrink: 0; display: flex; flex-direction: column; gap: 10px; }
.cw-focus-main .cw-focus-tree { flex: 1; }
.cw-focus-main .cw-focus-head { flex-direction: column; align-items: stretch; }
.cw-focus-head { display: flex; align-items: center; gap: 14px; flex-shrink: 0; }
.cw-focus-desc { flex: 1; min-width: 0; font-size: 12px; line-height: 1.7; color: var(--color-text-secondary, #64748b); }
.cw-focus-desc b { color: var(--color-text, #1b1c1f); }
.cw-focus-opts { display: flex; align-items: center; gap: 14px; flex-shrink: 0; padding: 8px 12px; border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; background: var(--editor-panel-bg, #fff); font-size: 12.5px; }
.cw-focus-opt-label { color: var(--color-text-secondary, #64748b); }
.cw-focus-opts small { display: block; font-size: 11px; color: var(--color-text-muted, #9aa0a6); margin-top: 1px; }
.cw-focus-count { margin-left: auto; font-size: 12px; color: #3e7bfa; white-space: nowrap; }
.cw-focus-tree { flex: 1; min-height: 0; overflow-y: auto; border: 1px solid var(--editor-border, #e2e8f0); border-radius: 8px; background: var(--editor-panel-bg, #fff); padding: 6px 8px 12px; }
.cw-ai-on { color: var(--color-brand, #16a34a) !important; border-color: var(--color-brand, #16a34a) !important; }

/* 分析等待态(数据驱动重点文件步骤) */
.cw-pstage.wait .cw-pdot, .cw-rstage.wait .cw-rdot { background: transparent; border: 2px solid #3e7bfa; width: 7px; height: 7px; }
.cw-pstage.wait, .cw-rstage.wait { color: #3e7bfa; }
</style>
