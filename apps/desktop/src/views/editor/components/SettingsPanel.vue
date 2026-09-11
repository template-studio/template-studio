<template>
  <div class="settings-pane" :style="{ width: paneWidth + 'px' }">
    <div class="pane-head">设置</div>
    <div class="pane-body">
      <a-tabs v-model:activeKey="activeTab" size="small" class="settings-tabs">
        <!-- 编辑器设置 -->
        <a-tab-pane key="editor-settings" tab="编辑器">
          <div class="tab-content-scroll">
            <div class="tab-content">
              <div class="set-section">编辑器</div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">自动保存</span>
                  <a-switch v-model:checked="localSettings.autoSave.enabled" size="small" />
                </div>
                <div class="set-desc">文件修改后自动保存</div>
              </div>
              <div v-if="localSettings.autoSave.enabled" class="set-row">
                <div class="set-head"><span class="set-title">自动保存间隔</span></div>
                <div class="set-desc">自动保存的时间间隔（秒）</div>
                <div class="set-ctl">
                  <a-input-number
                    v-model:value="localSettings.autoSave.interval"
                    :min="5" :max="300" :step="5" style="width: 100%" placeholder="30" size="small"
                  />
                  <span class="set-unit">秒</span>
                </div>
              </div>
              <div class="set-row">
                <div class="set-head"><span class="set-title">字体大小</span></div>
                <div class="set-desc">编辑器字体大小</div>
                <div class="set-ctl">
                  <a-input-number
                    v-model:value="localSettings.editor.fontSize"
                    :min="10" :max="24" style="width: 100%" placeholder="14" size="small"
                  />
                  <span class="set-unit">px</span>
                </div>
              </div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">显示行号</span>
                  <a-switch v-model:checked="localSettings.editor.lineNumbers" size="small" />
                </div>
                <div class="set-desc">在编辑器中显示行号</div>
              </div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">自动换行</span>
                  <a-switch v-model:checked="localSettings.editor.wordWrap" size="small" />
                </div>
                <div class="set-desc">长行自动换行显示</div>
              </div>

              <div class="set-section">界面</div>
              <div class="set-row">
                <div class="set-head"><span class="set-title">主题</span></div>
                <div class="set-desc">选择编辑器主题</div>
                <div class="set-ctl">
                  <a-select
                    v-model:value="localSettings.interface.theme"
                    :options="themeOptions" style="width: 100%" size="small"
                  />
                </div>
              </div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">启动时恢复面板布局</span>
                  <a-switch v-model:checked="localSettings.interface.restoreLayout" size="small" />
                </div>
                <div class="set-desc">记住并恢复面板的大小和位置</div>
              </div>

              <div class="set-section">预览</div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">实时预览</span>
                  <a-switch v-model:checked="localSettings.preview.realtime" size="small" />
                </div>
                <div class="set-desc">编辑时自动更新预览</div>
              </div>
              <div v-if="localSettings.preview.realtime" class="set-row">
                <div class="set-head"><span class="set-title">预览延迟</span></div>
                <div class="set-desc">输入停止后延迟更新预览（毫秒）</div>
                <div class="set-ctl">
                  <a-input-number
                    v-model:value="localSettings.preview.debounceDelay"
                    :min="100" :max="5000" :step="100" style="width: 100%" placeholder="500" size="small"
                  />
                  <span class="set-unit">ms</span>
                </div>
              </div>
            </div>
          </div>
        </a-tab-pane>
        <!-- 引擎管理 -->
        <a-tab-pane key="engine" tab="引擎">
          <div class="tab-content-scroll">
            <div class="tab-content">
              <div class="set-section">引擎状态</div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">当前引擎</span>
                  <a-tag :color="engineState.isUsingWasm ? 'success' : 'processing'" class="set-tag">
                    {{ engineState.currentEngine || '未初始化' }}{{ engineState.isUsingWasm ? ' · 离线可用' : '' }}
                  </a-tag>
                </div>
                <div class="set-desc">正在使用的渲染引擎</div>
              </div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">引擎版本</span>
                  <span class="set-value">{{ engineState.version || '-' }}</span>
                </div>
                <div class="set-desc">当前引擎的版本号</div>
              </div>
              <div class="set-row">
                <div class="set-head">
                  <span class="set-title">初始化状态</span>
                  <span>
                    <a-tag v-if="engineState.isLoading" color="warning">加载中...</a-tag>
                    <a-tag v-else-if="engineState.isReady" color="success">已就绪</a-tag>
                    <a-tag v-else color="error">未初始化</a-tag>
                  </span>
                </div>
                <div class="set-desc">引擎是否已准备就绪</div>
              </div>
              <div v-if="engineState.error" class="set-row">
                <div class="set-head"><span class="set-title err">错误信息</span></div>
                <div class="set-desc err">{{ engineState.error }}</div>
              </div>

              <div class="set-section">引擎切换</div>
              <div class="set-row">
                <a-radio-group
                  v-model:value="selectedEngine"
                  :disabled="engineState.isLoading"
                  class="set-radios"
                >
                  <a-radio value="wasm">
                    本地引擎
                    <a-tag v-if="engineState.wasmReady" color="success" class="set-tag">可用</a-tag>
                    <a-tag v-else color="warning" class="set-tag">不可用</a-tag>
                  </a-radio>
                  <a-radio value="backend">
                    后端引擎
                    <a-tag v-if="engineState.backendReady" color="success" class="set-tag">可用</a-tag>
                    <a-tag v-else color="warning" class="set-tag">不可用</a-tag>
                  </a-radio>
                </a-radio-group>
                <div class="set-desc">本地引擎（Tauri 原生）支持离线渲染，后端引擎需要网络连接</div>
              </div>

              <div class="set-section">引擎详情</div>
              <a-spin :spinning="engineState.isLoadingInfo">
                <div v-if="engineInfo" class="engine-info">
                  <div class="info-row">
                    <span class="info-label">版本号：</span>
                    <span class="info-value">{{ engineInfo.version || '-' }}</span>
                  </div>
                  <div class="info-row">
                    <span class="info-label">构建时间：</span>
                    <span class="info-value">{{ engineInfo.buildTime || '-' }}</span>
                  </div>
                  <div v-if="engineInfo.filters?.length" class="info-row">
                    <span class="info-label">内置过滤器：</span>
                    <div class="info-tags">
                      <a-tag v-for="filter in engineInfo.filters" :key="filter" size="small" style="margin: 2px">
                        {{ filter }}
                      </a-tag>
                    </div>
                  </div>
                  <div v-if="engineInfo.functions?.length" class="info-row">
                    <span class="info-label">内置函数：</span>
                    <div class="info-tags">
                      <a-tag v-for="func in engineInfo.functions" :key="func" size="small" color="processing" style="margin: 2px">
                        {{ func }}
                      </a-tag>
                    </div>
                  </div>
                </div>
                <a-empty v-else description="暂无引擎信息" />
              </a-spin>
            </div>
          </div>
        </a-tab-pane>
        <!-- 备份与恢复 -->
        <a-tab-pane key="backup" tab="备份">
          <div class="tab-content-scroll">
            <div class="tab-content">
              <div class="set-section">创建备份</div>
              <div class="set-row">
                <div class="set-desc">将当前模板完整导出为 .tsbk 备份文件，包含文件、变量定义、测试数据和文件条件</div>
                <div class="set-ctl">
                  <a-button
                    type="primary" block size="small"
                    :loading="backupState.isCreating" :disabled="!templateId"
                    @click="handleCreateBackup"
                  >创建备份</a-button>
                </div>
                <a-progress
                  v-if="backupState.isCreating && backupState.progress > 0"
                  :percent="backupState.progress / 100" :status="backupState.progressStatus" size="small"
                >
                  <template #format="{ percent }">
                    {{ backupState.progressMessage || `${Math.round(percent * 100)}%` }}
                  </template>
                </a-progress>
              </div>

              <div class="set-section">恢复备份</div>
              <div class="set-row">
                <div class="set-desc">从 .tsbk 备份文件恢复模板，将覆盖当前模板的所有内容</div>
                <div class="set-ctl">
                  <a-upload
                    :custom-request="handleBackupFileSelect" :show-upload-list="false"
                    accept=".tsbk" :disabled="!templateId" style="width: 100%"
                  >
                    <a-button block size="small" :disabled="!templateId">选择备份文件</a-button>
                  </a-upload>
                </div>
              </div>

              <!-- 备份预览 -->
              <div v-if="backupState.preview" class="backup-preview">
                <a-descriptions label-placement="left" :column="1" bordered size="small">
                  <a-descriptions-item label="模板名称">
                    {{ backupState.preview.templateName }}
                  </a-descriptions-item>
                  <a-descriptions-item label="文件数量">
                    {{ backupState.preview.fileCount }} 个
                  </a-descriptions-item>
                  <a-descriptions-item label="备份时间">
                    {{ formatBackupTime(backupState.preview.manifest.createdAt) }}
                  </a-descriptions-item>
                  <a-descriptions-item label="校验状态">
                    <a-tag :color="backupState.preview.checksumValid ? 'success' : 'error'" size="small">
                      {{ backupState.preview.checksumValid ? '校验通过' : '校验失败' }}
                    </a-tag>
                  </a-descriptions-item>
                  <a-descriptions-item label="包含变量">
                    <a-tag :color="backupState.preview.hasVariables ? 'success' : 'default'" size="small">
                      {{ backupState.preview.hasVariables ? '是' : '否' }}
                    </a-tag>
                  </a-descriptions-item>
                  <a-descriptions-item label="包含测试数据">
                    <a-tag :color="backupState.preview.hasTestData ? 'success' : 'default'" size="small">
                      {{ backupState.preview.hasTestData ? '是' : '否' }}
                    </a-tag>
                  </a-descriptions-item>
                </a-descriptions>

                <a-alert type="warning" style="margin-top: 10px" message="警告">
                  <template #description>
                    恢复备份将覆盖当前模板的所有内容，此操作不可撤销。请确认备份文件来源可信。
                  </template>
                </a-alert>

                <a-progress
                  v-if="backupState.isRestoring && backupState.restoreProgress > 0"
                  :percent="backupState.restoreProgress / 100" :status="backupState.restoreStatus" size="small"
                  style="margin-top: 10px"
                >
                  <template #format="{ percent }">
                    {{ backupState.restoreMessage || `${Math.round(percent * 100)}%` }}
                  </template>
                </a-progress>

                <div class="set-ctl">
                  <a-button danger size="small" :loading="backupState.isRestoring" :disabled="!backupState.preview.checksumValid" @click="handleRestoreBackup">
                    确认恢复
                  </a-button>
                  <a-button size="small" @click="clearBackupPreview">取消</a-button>
                </div>
              </div>

              <!-- 恢复结果 -->
              <div v-if="backupState.restoreResult" class="backup-preview">
                <a-result
                  :status="backupState.restoreResult.success ? 'success' : 'error'"
                  :title="backupState.restoreResult.success ? '恢复成功' : '恢复失败'"
                  :sub-title="backupState.restoreResult.error || '模板已成功从备份恢复'"
                >
                  <template v-if="backupState.restoreResult.stats" #extra>
                    <a-descriptions label-placement="left" :column="1" size="small">
                      <a-descriptions-item label="文件恢复">
                        {{ backupState.restoreResult.stats.filesRestored }} 个
                      </a-descriptions-item>
                      <a-descriptions-item label="变量恢复">
                        {{ backupState.restoreResult.stats.variablesRestored }} 个
                      </a-descriptions-item>
                      <a-descriptions-item label="条件恢复">
                        {{ backupState.restoreResult.stats.conditionsRestored }} 个
                      </a-descriptions-item>
                      <a-descriptions-item label="测试数据">
                        {{ backupState.restoreResult.stats.testDataRestored ? '已恢复' : '未恢复' }}
                      </a-descriptions-item>
                    </a-descriptions>
                  </template>
                </a-result>
                <div class="set-ctl">
                  <a-button size="small" @click="clearRestoreResult">关闭</a-button>
                </div>
              </div>

              <div class="set-section">关于 .tsbk 格式</div>
              <a-collapse>
                <a-collapse-panel key="structure" header="备份文件结构">
                  <pre class="format-pre">{{ backupFormatInfo }}</pre>
                </a-collapse-panel>
                <a-collapse-panel key="security" header="安全说明">
                  <span>
                    备份文件使用 SHA256 校验和防止篡改。如果校验失败，系统将拒绝恢复备份。
                    请确保备份文件来自可信来源。
                  </span>
                </a-collapse-panel>
              </a-collapse>
            </div>
          </div>
        </a-tab-pane>
            </a-tabs>
    </div>

    <!-- 底部按钮区域(随当前 tab 切换) -->
    <div class="pane-foot">
      <template v-if="activeTab === 'editor-settings'">
        <a-button size="small" @click="resetToDefaults">恢复默认</a-button>
        <a-button size="small" type="primary" @click="handleSave">保存设置</a-button>
      </template>
      <template v-else-if="activeTab === 'engine'">
        <a-button size="small" @click="refreshEngineStatus" :loading="engineState.isLoading">
          刷新状态
        </a-button>
        <a-button size="small" @click="handleClearCache" :disabled="!engineState.isUsingWasm">
          清除缓存
        </a-button>
        <a-button size="small" type="primary" @click="refreshEngineInfo" :loading="engineState.isLoadingInfo">
          获取引擎信息
        </a-button>
      </template>
    </div>

    <!-- 右缘拖拽调宽(与预览面板同款交互) -->
    <div class="pane-resize" :class="{ resizing: isResizing }" @mousedown="startResize"></div>
  </div>
</template>

<script setup lang="ts">
  // 编辑器设置侧栏面板(#200)：原 80vw 高级设置抽屉改造为 VSCode 式侧栏视图。
  // 三个 tab(编辑器设置/引擎管理/备份与恢复)常驻挂载,由父组件 activeView 控制显隐。
  import { ref, watch, computed, reactive, onMounted } from 'vue';
  import { message, Modal } from 'ant-design-vue';
  import { useRenderService } from '@/composables/useRenderService';
  import {
    createBackup,
    previewBackup,
    restoreBackup,
    type BackupPreviewResponse,
    type RestoreBackupResponse,
  } from '@/api/editor/backup';
  import { saveAs } from 'file-saver';

  const props = defineProps({
    settings: {
      type: Object,
      default: () => ({}),
    },
    templateId: {
      type: [Number, String],
      default: null,
    },
  });

  const emit = defineEmits(['save-settings', 'backup-complete', 'restore-complete']);

  const activeTab = ref('editor-settings');

  // 面板宽度:默认 360,可拖拽(280-560),记忆到 localStorage
  const PANE_MIN = 280;
  const PANE_MAX = 560;
  const clampWidth = (w) => Math.min(PANE_MAX, Math.max(PANE_MIN, w));
  const paneWidth = ref(clampWidth(parseInt(localStorage.getItem('editor-settings-pane-width'), 10) || 360));
  const isResizing = ref(false);
  let resizeStartX = 0;
  let resizeStartWidth = 0;

  const startResize = (e) => {
    e.preventDefault();
    isResizing.value = true;
    resizeStartX = e.clientX;
    resizeStartWidth = paneWidth.value;
    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  };

  const handleResize = (e) => {
    if (!isResizing.value) return;
    // 左锚定面板:手柄右移(clientX 增大)加宽
    paneWidth.value = clampWidth(resizeStartWidth + (e.clientX - resizeStartX));
  };

  const stopResize = () => {
    isResizing.value = false;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    localStorage.setItem('editor-settings-pane-width', String(paneWidth.value));
  };

  // 引擎管理相关
  const {
    isReady: engineIsReady,
    currentEngine,
    engineVersion,
    isLoading: engineIsLoading,
    error: engineError,
    isUsingWasm,
    switchEngine,
    clearWasmCache,
    initialize: initEngine,
    getStatus,
  } = useRenderService({ autoInit: false });

  // 引擎状态
  const engineState = reactive({
    isReady: computed(() => engineIsReady.value),
    currentEngine: computed(() => currentEngine.value),
    version: computed(() => engineVersion.value),
    isLoading: computed(() => engineIsLoading.value),
    error: computed(() => engineError.value),
    isUsingWasm: computed(() => isUsingWasm.value),
    wasmReady: false,
    backendReady: false,
    isLoadingInfo: false,
  });

  // 选中的引擎
  const selectedEngine = ref('wasm');

  // 引擎信息
  const engineInfo = ref<{
    version?: string;
    buildTime?: string;
    filters?: string[];
    functions?: string[];
  } | null>(null);

  // =============== 备份恢复相关 ===============
  const backupState: any = reactive({
    // 创建备份状态
    isCreating: false,
    // 恢复备份状态
    isRestoring: false,
    // 预览数据
    preview: null as BackupPreviewResponse | null,
    selectedFile: null as File | null,
    // 恢复结果
    restoreResult: null as RestoreBackupResponse | null,
  });

  // 备份格式说明
  const backupFormatInfo = `template_backup.tsbk (ZIP 格式)
├── manifest.json          # 元数据 + 校验信息
├── template.json          # 模板基本信息
├── variables.json         # 变量定义
├── conditions.json        # 文件条件配置
├── testdata.json          # 测试数据
├── files/                 # 模板文件目录
│   ├── src/main.go
│   └── ...
└── .checksum              # 文件校验映射`;

  // 格式化备份时间
  function formatBackupTime(isoString: string): string {
    try {
      const date = new Date(isoString);
      return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return isoString;
    }
  }

  // 创建备份 - 调用后端 API
  async function handleCreateBackup() {
    if (!props.templateId) {
      message.error('模板ID不存在');
      return;
    }

    backupState.isCreating = true;

    try {
      const response = await createBackup({
        templateId: Number(props.templateId),
        includeTestData: true,
        includeConditions: true,
      });

      // response.data 是 Blob（axios 返回完整 response 对象）
      const blob = (response as any).data as Blob;
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
      const filename = `template_${props.templateId}_backup_${timestamp}.tsbk`;
      saveAs(blob, filename);

      message.success('备份创建成功');
      emit('backup-complete');
    } catch (error) {
      console.error('Backup failed:', error);
      message.error(error instanceof Error ? error.message : '备份创建失败');
    } finally {
      backupState.isCreating = false;
    }
  }

  // 选择备份文件 - 调用后端 API 预览
  async function handleBackupFileSelect({ file }: { file: any }) {
    const selectedFile = file.originFileObj || file;
    if (!selectedFile) return;

    // 验证文件扩展名
    if (!selectedFile.name.endsWith('.tsbk')) {
      message.error('请选择 .tsbk 格式的备份文件');
      return;
    }

    backupState.selectedFile = selectedFile;
    backupState.preview = null;
    backupState.restoreResult = null;

    // 预览备份 - 调用后端 API
    try {
      const response = await previewBackup(selectedFile);
      // API 返回格式: { code: 0, message: "OK", data: preview }
      backupState.preview = (response as any).data || response;

      if (!backupState.preview.checksumValid) {
        message.warning('备份文件校验失败，可能已被篡改');
      }
    } catch (error) {
      console.error('Preview failed:', error);
      message.error(error instanceof Error ? error.message : '预览备份文件失败');
    }
  }

  // 清除预览
  function clearBackupPreview() {
    backupState.preview = null;
    backupState.selectedFile = null;
    backupState.restoreResult = null;
  }

  // 恢复备份 - 调用后端 API
  async function handleRestoreBackup() {
    if (!props.templateId || !backupState.selectedFile) {
      message.error('缺少必要参数');
      return;
    }

    if (!backupState.preview?.checksumValid) {
      message.error('备份文件校验失败，无法恢复');
      return;
    }

    // 确认对话框
    Modal.confirm({
      title: '确认恢复',
      content: '恢复备份将覆盖当前模板的所有内容，此操作不可撤销。确定要继续吗？',
      okText: '确定恢复',
      cancelText: '取消',
      onOk: async () => {
        backupState.isRestoring = true;

        try {
          const response = await restoreBackup({
            templateId: Number(props.templateId),
            file: backupState.selectedFile!,
          });
          // API 返回格式: { code: 0, message: "...", data: result }
          const result = (response as any).data || response;
          backupState.restoreResult = result;

          if (result.success) {
            message.success('备份恢复成功');
            emit('restore-complete', result);
          } else {
            message.error(result.error || '备份恢复失败');
          }
        } catch (error) {
          console.error('Restore failed:', error);
          backupState.restoreResult = {
            success: false,
            error: error instanceof Error ? error.message : '未知错误',
          };
          message.error(error instanceof Error ? error.message : '备份恢复失败');
        } finally {
          backupState.isRestoring = false;
        }
      },
    });
  }

  // 清除恢复结果
  function clearRestoreResult() {
    backupState.restoreResult = null;
    backupState.preview = null;
    backupState.selectedFile = null;
  }

  // 刷新引擎状态
  async function refreshEngineStatus() {
    try {
      await initEngine();
      const status = getStatus();
      engineState.wasmReady = status.wasmReady;
      engineState.backendReady = status.backendReady;
      // 同步选中状态
      selectedEngine.value = ['WASM', 'Tauri'].includes(status.currentEngine) ? 'wasm' : 'backend';
    } catch (e) {
      console.error('Failed to refresh engine status:', e);
    }
  }

  // 切换引擎
  async function handleEngineSwitch(value: 'wasm' | 'backend') {
    const success = await switchEngine(value);
    if (success) {
      message.success(`已切换到 ${value === 'wasm' ? '本地' : '后端'} 引擎`);
    } else {
      message.error('引擎切换失败');
      // 恢复之前的选择
      selectedEngine.value = isUsingWasm.value ? 'wasm' : 'backend';
    }
  }

  // 清除缓存
  async function handleClearCache() {
    try {
      await clearWasmCache();
      message.success('缓存已清除');
    } catch (e) {
      message.error('清除缓存失败');
    }
  }

  // 获取引擎信息
  async function refreshEngineInfo() {
    engineState.isLoadingInfo = true;
    try {
      // 从 RenderService 获取引擎信息
      const { RenderService } = await import('@/services');
      const service = RenderService.getInstance();
      const info = await service.getCurrentEngineInfo();
      if (info) {
        engineInfo.value = {
          version: info.version,
          buildTime: info.buildTime,
          filters: info.filters,
          functions: info.functions,
        };
      }
    } catch (e) {
      console.error('Failed to get engine info:', e);
      message.error('获取引擎信息失败');
    } finally {
      engineState.isLoadingInfo = false;
    }
  }

  // 主题选项
  const themeOptions = [
    { label: '浅色主题', value: 'light' },
    { label: '深色主题', value: 'dark' },
    { label: '跟随系统', value: 'auto' },
  ];

  // 默认设置
  const defaultSettings = {
    autoSave: {
      enabled: true,
      interval: 30,
    },
    editor: {
      fontSize: 14,
      lineNumbers: true,
      wordWrap: true,
    },
    interface: {
      theme: 'light',
      restoreLayout: true,
    },
    preview: {
      realtime: true,
      debounceDelay: 500,
    },
  };

  // 本地设置副本：声明即带入默认值——面板常驻挂载，首渲染先于 onMounted，
  // 空 {} 会让模板访问 autoSave.enabled 直接抛错
  const localSettings = ref<any>(JSON.parse(JSON.stringify(defaultSettings)));

  // 初始化设置
  const initSettings = () => {
    localSettings.value = JSON.parse(
      JSON.stringify({
        ...defaultSettings,
        ...props.settings,
      })
    );
  };

  // 重置为默认值
  const resetToDefaults = () => {
    localSettings.value = JSON.parse(JSON.stringify(defaultSettings));
    message.success('已恢复为默认设置');
  };

  // 保存设置
  const handleSave = () => {
    emit('save-settings', JSON.parse(JSON.stringify(localSettings.value)));
    message.success('设置已保存');
  };

  // 侧栏常驻面板：挂载即初始化设置副本
  onMounted(() => {
    initSettings();
  });

  // 监听标签切换（引擎管理需要实时状态）
  watch(activeTab, (newTab) => {
    if (newTab === 'engine') {
      refreshEngineStatus();
    }
  });

  watch(() => props.settings, initSettings, { deep: true });

  // 暴露方法给父组件
  const openTab = (tabName) => {
    activeTab.value = tabName;
  };

  defineExpose({
    openTab,
  });
</script>

<style scoped>
  /* 侧栏设置面板(原 80vw 抽屉,#200 改造):宽度由内联 style 控制(可拖拽) */
  .settings-pane {
    flex-shrink: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--editor-panel-bg, #fff);
    position: relative;
  }

  /* 右缘拖拽调宽手柄 */
  .pane-resize {
    position: absolute;
    right: -4px;
    top: 0;
    width: 8px;
    height: 100%;
    cursor: col-resize;
    z-index: 10;
    user-select: none;
    touch-action: none;
    background: transparent;
    transition: background 0.2s;
  }

  .pane-resize:hover,
  .pane-resize.resizing {
    background: rgba(34, 197, 94, 0.18);
  }

  .pane-head {
    height: 34px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    padding: 0 14px;
    border-bottom: 1px solid var(--editor-border, #f0f0ee);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: var(--color-text, #1b1c1f);
  }

  .pane-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* Tab:顶部小号(窄栏放不下左侧竖排) */
  .settings-tabs {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  :deep(.ant-tabs-content-holder) {
    flex: 1 !important;
    min-width: 0;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  :deep(.ant-tabs-content) {
    height: 100% !important;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  :deep(.ant-tabs-tabpane) {
    width: 100%;
    height: 100%;
    overflow: auto !important;
  }

  :deep(.ant-tabs-nav) {
    padding: 0 8px;
    margin: 0 0 6px;
  }

  :deep(.ant-tabs-tab) {
    padding: 6px 0 !important;
    margin: 0 10px 0 0 !important;
    font-size: 12.5px;
  }

  .tab-content-scroll {
    height: 100%;
    overflow-y: auto;
  }

  .tab-content {
    padding: 4px 0 16px;
    width: 100%;
    box-sizing: border-box;
  }

  /* VSCode 式窄栏设置行(#200 补4):分组标题 + 行(开关右置/输入下挂) */
  .set-section {
    padding: 12px 14px 4px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.5px;
    color: var(--editor-muted, #94a3b8);
  }

  .set-section:first-child {
    padding-top: 6px;
  }

  .set-row {
    padding: 6px 14px 8px;
    border-radius: 6px;
  }

  .set-row:hover {
    background: var(--editor-inset-bg, #f8fafc);
  }

  .set-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 24px;
  }

  .set-title {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--editor-primary, #1e293b);
    line-height: 1.4;
  }

  .set-title.err {
    color: #d03050;
  }

  .set-desc {
    font-size: 11.5px;
    color: var(--editor-muted, #94a3b8);
    line-height: 1.45;
    margin-top: 2px;
  }

  .set-desc.err {
    color: #d03050;
  }

  .set-value {
    font-size: 12px;
    color: var(--editor-primary, #1e293b);
  }

  .set-ctl {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
  }

  .set-unit {
    flex-shrink: 0;
    font-size: 11.5px;
    color: var(--editor-muted, #94a3b8);
  }

  .set-tag {
    font-size: 11px;
    padding: 0 6px;
    line-height: 18px;
  }

  .set-radios {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: flex-start;
  }

  .format-pre {
    background: var(--editor-inset-bg, #f5f5f5);
    padding: 10px;
    border-radius: 4px;
    margin: 0;
    font-size: 11.5px;
    white-space: pre;
    overflow-x: auto;
  }

  /* 底部按钮区域(随 tab 切换,窄栏可换行) */
  .pane-foot {
    flex-shrink: 0;
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 6px;
    padding: 8px 10px;
    border-top: 1px solid var(--editor-border, #e2e8f0);
    background: var(--editor-panel-bg, #fff);
  }

  /* 引擎管理样式 */
  .engine-info {
    padding: 8px 0;
  }

  .info-row {
    display: flex;
    align-items: flex-start;
    padding: 12px 0;
    border-bottom: 1px solid var(--editor-border, #f1f5f9);
    transition: background 0.2s ease;
  }

  .info-row:hover {
    background: var(--editor-inset-bg, #fafbfc);
    margin: 0 -12px;
    padding: 12px;
  }

  .info-row:last-child {
    border-bottom: none;
  }

  .info-label {
    flex-shrink: 0;
    width: 90px;
    font-size: 13px;
    color: var(--editor-muted, #64748b);
    font-weight: 500;
  }

  .info-value {
    font-size: 13px;
    color: var(--editor-primary, #1e293b);
  }

  .info-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .info-tags :deep(.ant-tag) {
    border-radius: 4px;
    font-size: 11px;
  }

  /* 备份恢复样式 */
  .backup-preview {
    margin-top: 12px;
    padding: 12px;
    background: var(--editor-inset-bg, #f8fafc);
    border-radius: 8px;
    border: 1px solid var(--editor-border, #e2e8f0);
  }

  .backup-preview :deep(.ant-descriptions) {
    background: var(--editor-panel-bg, #fff);
    border-radius: 8px;
    overflow: hidden;
  }

  .backup-preview :deep(.ant-alert) {
    border-radius: 8px;
  }
</style>
