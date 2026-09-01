<template>
  <a-drawer
    v-model:open="visible"
    title="高级设置"
    :width="'80vw'"
    placement="right"
    :body-style="{ padding: '0' }"
    class="advanced-drawer"
  >
    <div class="drawer-content">
      <a-tabs
        v-model:activeKey="activeTab"
        tab-position="left"
        :tab-bar-style="{ minWidth: '120px' }"
      >
        <!-- 编辑器设置 -->
        <a-tab-pane key="editor-settings" tab="编辑器设置">
          <div class="tab-content-scroll">
            <div class="tab-content">
              <!-- 编辑器卡片 -->
              <a-card title="编辑器" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>自动保存</span>
                    <span class="setting-description">文件修改后自动保存</span>
                  </div>
                  <div class="setting-control">
                    <a-switch v-model:checked="localSettings.autoSave.enabled" />
                  </div>
                </div>

                <div v-if="localSettings.autoSave.enabled" class="setting-item">
                  <div class="setting-label">
                    <span>自动保存间隔</span>
                    <span class="setting-description">自动保存的时间间隔（秒）</span>
                  </div>
                  <div class="setting-control">
                    <a-input-number
                      v-model:value="localSettings.autoSave.interval"
                      :min="5"
                      :max="300"
                      :step="5"
                      style="width: 120px"
                      placeholder="30"
                    />
                    <span style="margin-left: 8px; color: var(--editor-muted, #666)">秒</span>
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-label">
                    <span>字体大小</span>
                    <span class="setting-description">编辑器字体大小</span>
                  </div>
                  <div class="setting-control">
                    <a-input-number
                      v-model:value="localSettings.editor.fontSize"
                      :min="10"
                      :max="24"
                      style="width: 120px"
                      placeholder="14"
                    />
                    <span style="margin-left: 8px; color: var(--editor-muted, #666)">px</span>
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-label">
                    <span>显示行号</span>
                    <span class="setting-description">在编辑器中显示行号</span>
                  </div>
                  <div class="setting-control">
                    <a-switch v-model:checked="localSettings.editor.lineNumbers" />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-label">
                    <span>自动换行</span>
                    <span class="setting-description">长行自动换行显示</span>
                  </div>
                  <div class="setting-control">
                    <a-switch v-model:checked="localSettings.editor.wordWrap" />
                  </div>
                </div>
              </a-card>

              <!-- 界面卡片 -->
              <a-card title="界面" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>主题</span>
                    <span class="setting-description">选择编辑器主题</span>
                  </div>
                  <div class="setting-control">
                    <a-select
                      v-model:value="localSettings.interface.theme"
                      :options="themeOptions"
                      style="width: 150px"
                    />
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-label">
                    <span>启动时恢复面板布局</span>
                    <span class="setting-description">记住并恢复面板的大小和位置</span>
                  </div>
                  <div class="setting-control">
                    <a-switch v-model:checked="localSettings.interface.restoreLayout" />
                  </div>
                </div>
              </a-card>

              <!-- 预览卡片 -->
              <a-card title="预览" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>实时预览</span>
                    <span class="setting-description">编辑时自动更新预览</span>
                  </div>
                  <div class="setting-control">
                    <a-switch v-model:checked="localSettings.preview.realtime" />
                  </div>
                </div>

                <div v-if="localSettings.preview.realtime" class="setting-item">
                  <div class="setting-label">
                    <span>预览延迟</span>
                    <span class="setting-description">输入停止后延迟更新预览（毫秒）</span>
                  </div>
                  <div class="setting-control">
                    <a-input-number
                      v-model:value="localSettings.preview.debounceDelay"
                      :min="100"
                      :max="5000"
                      :step="100"
                      style="width: 120px"
                      placeholder="500"
                    />
                    <span style="margin-left: 8px; color: var(--editor-muted, #666)">ms</span>
                  </div>
                </div>
              </a-card>
            </div>
          </div>
        </a-tab-pane>

        <!-- 引擎管理 -->
        <a-tab-pane key="engine" tab="引擎管理">
          <div class="tab-content-scroll">
            <div class="tab-content">
              <!-- 引擎状态卡片 -->
              <a-card title="引擎状态" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>当前引擎</span>
                    <span class="setting-description">正在使用的渲染引擎</span>
                  </div>
                  <div class="setting-control">
                    <a-tag :color="engineState.isUsingWasm ? 'success' : 'processing'" style="font-size: 14px; padding: 4px 12px">
                      {{ engineState.currentEngine || '未初始化' }}
                      <template v-if="engineState.isUsingWasm"> (离线可用)</template>
                    </a-tag>
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-label">
                    <span>引擎版本</span>
                    <span class="setting-description">当前引擎的版本号</span>
                  </div>
                  <div class="setting-control">
                    <span>{{ engineState.version || '-' }}</span>
                  </div>
                </div>

                <div class="setting-item">
                  <div class="setting-label">
                    <span>初始化状态</span>
                    <span class="setting-description">引擎是否已准备就绪</span>
                  </div>
                  <div class="setting-control">
                    <a-tag v-if="engineState.isLoading" color="warning">
                      <template #icon>
                        <svg viewBox="0 0 24 24" width="14" height="14" style="margin-right: 4px">
                          <path
                            fill="currentColor"
                            d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8z"
                          />
                        </svg>
                      </template>
                      加载中...
                    </a-tag>
                    <a-tag v-else-if="engineState.isReady" color="success">已就绪</a-tag>
                    <a-tag v-else color="error">未初始化</a-tag>
                  </div>
                </div>

                <div v-if="engineState.error" class="setting-item">
                  <div class="setting-label">
                    <span style="color: #d03050">错误信息</span>
                    <span class="setting-description" style="color: #d03050">{{
                      engineState.error
                    }}</span>
                  </div>
                </div>
              </a-card>

              <!-- 引擎切换卡片 -->
              <a-card title="引擎切换" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>选择引擎</span>
                    <span class="setting-description"
                      >本地引擎（Tauri 原生）支持离线渲染，后端引擎需要网络连接</span
                    >
                  </div>
                  <div class="setting-control">
                    <a-radio-group
                      v-model:value="selectedEngine"
                      :disabled="engineState.isLoading"
                      @change="(e) => handleEngineSwitch(e.target.value)"
                    >
                      <a-space>
                        <a-radio value="wasm">
                          <a-space align="center" :size="6">
                            <span>本地引擎</span>
                            <a-tag v-if="engineState.wasmReady" color="success" size="small"
                              >可用</a-tag
                            >
                            <a-tag v-else color="warning" size="small">不可用</a-tag>
                          </a-space>
                        </a-radio>
                        <a-radio value="backend">
                          <a-space align="center" :size="6">
                            <span>后端引擎</span>
                            <a-tag v-if="engineState.backendReady" color="success" size="small"
                              >可用</a-tag
                            >
                            <a-tag v-else color="warning" size="small">不可用</a-tag>
                          </a-space>
                        </a-radio>
                      </a-space>
                    </a-radio-group>
                  </div>
                </div>
              </a-card>

              <!-- 引擎信息卡片 -->
              <a-card title="引擎详情" :bordered="true" class="settings-card">
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
                        <a-tag
                          v-for="filter in engineInfo.filters"
                          :key="filter"
                          size="small"
                          style="margin: 2px"
                        >
                          {{ filter }}
                        </a-tag>
                      </div>
                    </div>
                    <div v-if="engineInfo.functions?.length" class="info-row">
                      <span class="info-label">内置函数：</span>
                      <div class="info-tags">
                        <a-tag
                          v-for="func in engineInfo.functions"
                          :key="func"
                          size="small"
                          color="processing"
                          style="margin: 2px"
                        >
                          {{ func }}
                        </a-tag>
                      </div>
                    </div>
                  </div>
                  <a-empty v-else description="暂无引擎信息" />
                </a-spin>
              </a-card>
            </div>
          </div>
        </a-tab-pane>

        <!-- 备份与恢复 -->
        <a-tab-pane key="backup" tab="备份与恢复">
          <div class="tab-content-scroll">
            <div class="tab-content">
              <!-- 备份操作卡片 -->
              <a-card title="创建备份" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>导出模板备份</span>
                    <span class="setting-description">
                      将当前模板完整导出为 .tsbk 备份文件，包含文件、变量定义、测试数据和文件条件
                    </span>
                  </div>
                  <div class="setting-control">
                    <a-button
                      type="primary"
                      :loading="backupState.isCreating"
                      :disabled="!templateId"
                      @click="handleCreateBackup"
                    >
                      <template #icon>
                        <svg viewBox="0 0 24 24" width="18" height="18">
                          <path
                            fill="currentColor"
                            d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6 .67l2.59-2.58L17 11.5l-5 5-5-5 1.41-1.41L11 12.67V3h2v9.67z"
                          />
                        </svg>
                      </template>
                      创建备份
                    </a-button>
                  </div>
                </div>

                <a-progress
                  v-if="backupState.isCreating && backupState.progress > 0"
                  :percent="backupState.progress / 100"
                  :status="backupState.progressStatus"
                  style="margin-top: 16px"
                >
                  <template #format="{ percent }">
                    {{ backupState.progressMessage || `${Math.round(percent * 100)}%` }}
                  </template>
                </a-progress>
              </a-card>

              <!-- 恢复操作卡片 -->
              <a-card title="恢复备份" :bordered="true" class="settings-card">
                <div class="setting-item">
                  <div class="setting-label">
                    <span>导入备份文件</span>
                    <span class="setting-description">
                      从 .tsbk 备份文件恢复模板，将覆盖当前模板的所有内容
                    </span>
                  </div>
                  <div class="setting-control">
                    <a-upload
                      :custom-request="handleBackupFileSelect"
                      :show-upload-list="false"
                      accept=".tsbk"
                      :disabled="!templateId"
                    >
                      <a-button :disabled="!templateId">
                        <template #icon>
                          <svg viewBox="0 0 24 24" width="18" height="18">
                            <path
                              fill="currentColor"
                              d="M19 12v7H5v-7H3v7c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2zm-6-.67l2.59 2.58L17 12.5l-5-5-5 5 1.41 1.41L11 11.33V21h2v-9.67z"
                            />
                          </svg>
                        </template>
                        选择备份文件
                      </a-button>
                    </a-upload>
                  </div>
                </div>

                <!-- 备份预览 -->
                <div v-if="backupState.preview" class="backup-preview">
                  <a-divider style="margin: 16px 0" />
                  <a-descriptions label-placement="left" :column="2" bordered size="small">
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
                      <a-tag
                        :color="backupState.preview.checksumValid ? 'success' : 'error'"
                        size="small"
                      >
                        {{ backupState.preview.checksumValid ? '校验通过' : '校验失败' }}
                      </a-tag>
                    </a-descriptions-item>
                    <a-descriptions-item label="包含变量">
                      <a-tag
                        :color="backupState.preview.hasVariables ? 'success' : 'default'"
                        size="small"
                      >
                        {{ backupState.preview.hasVariables ? '是' : '否' }}
                      </a-tag>
                    </a-descriptions-item>
                    <a-descriptions-item label="包含测试数据">
                      <a-tag
                        :color="backupState.preview.hasTestData ? 'success' : 'default'"
                        size="small"
                      >
                        {{ backupState.preview.hasTestData ? '是' : '否' }}
                      </a-tag>
                    </a-descriptions-item>
                  </a-descriptions>

                  <a-alert type="warning" style="margin-top: 16px" message="警告">
                    <template #description>
                      恢复备份将覆盖当前模板的所有内容，此操作不可撤销。请确认备份文件来源可信。
                    </template>
                  </a-alert>

                  <a-progress
                    v-if="backupState.isRestoring && backupState.restoreProgress > 0"
                    :percent="backupState.restoreProgress / 100"
                    :status="backupState.restoreStatus"
                    style="margin-top: 16px"
                  >
                    <template #format="{ percent }">
                      {{ backupState.restoreMessage || `${Math.round(percent * 100)}%` }}
                    </template>
                  </a-progress>

                  <div style="margin-top: 16px; display: flex; gap: 12px">
                    <a-button
                      danger
                      :loading="backupState.isRestoring"
                      :disabled="!backupState.preview.checksumValid"
                      @click="handleRestoreBackup"
                    >
                      确认恢复
                    </a-button>
                    <a-button @click="clearBackupPreview"> 取消 </a-button>
                  </div>
                </div>
              </a-card>

              <!-- 恢复结果 -->
              <a-card
                v-if="backupState.restoreResult"
                title="恢复结果"
                :bordered="true"
                class="settings-card"
              >
                <a-result
                  :status="backupState.restoreResult.success ? 'success' : 'error'"
                  :title="backupState.restoreResult.success ? '恢复成功' : '恢复失败'"
                  :sub-title="backupState.restoreResult.error || '模板已成功从备份恢复'"
                >
                  <template v-if="backupState.restoreResult.stats" #extra>
                    <a-descriptions label-placement="left" :column="2" size="small">
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
                        {{
                          backupState.restoreResult.stats.testDataRestored ? '已恢复' : '未恢复'
                        }}
                      </a-descriptions-item>
                    </a-descriptions>
                  </template>
                </a-result>
                <div style="margin-top: 16px">
                  <a-button @click="clearRestoreResult"> 关闭 </a-button>
                </div>
              </a-card>

              <!-- 备份格式说明 -->
              <a-card title="关于 .tsbk 格式" :bordered="true" class="settings-card">
                <a-collapse>
                  <a-collapse-panel key="structure" header="备份文件结构">
                    <pre style="background: var(--editor-inset-bg, #f5f5f5); padding: 12px; border-radius: 4px; margin: 0; font-size: 13px">{{ backupFormatInfo }}</pre>
                  </a-collapse-panel>
                  <a-collapse-panel key="security" header="安全说明">
                    <span>
                      备份文件使用 SHA256 校验和防止篡改。如果校验失败，系统将拒绝恢复备份。
                      请确保备份文件来自可信来源。
                    </span>
                  </a-collapse-panel>
                </a-collapse>
              </a-card>
            </div>
          </div>
        </a-tab-pane>
      </a-tabs>
    </div>

    <!-- 底部按钮区域 -->
    <template #footer>
      <div class="drawer-footer">
        <template v-if="activeTab === 'editor-settings'">
          <a-button @click="resetToDefaults">恢复默认</a-button>
          <a-button type="primary" @click="handleSave">保存设置</a-button>
        </template>
        <template v-else-if="activeTab === 'engine'">
          <a-button @click="refreshEngineStatus" :loading="engineState.isLoading">
            刷新状态
          </a-button>
          <a-button @click="handleClearCache" :disabled="!engineState.isUsingWasm">
            清除缓存
          </a-button>
          <a-button type="primary" @click="refreshEngineInfo" :loading="engineState.isLoadingInfo">
            获取引擎信息
          </a-button>
        </template>
      </div>
    </template>
  </a-drawer>
</template>

<script setup lang="ts">
  import { ref, watch, computed, reactive } from 'vue';
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
    show: {
      type: Boolean,
      default: false,
    },
    settings: {
      type: Object,
      default: () => ({}),
    },
    templateId: {
      type: [Number, String],
      default: null,
    },
  });

  const emit = defineEmits(['update:show', 'save-settings', 'backup-complete', 'restore-complete']);

  const visible = ref(props.show);
  const activeTab = ref('editor-settings');

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

  // 本地设置副本
  const localSettings = ref<any>({});

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

  watch(
    () => props.show,
    (newVal) => {
      visible.value = newVal;
      if (newVal) {
        initSettings();
        // 如果是引擎管理标签，刷新状态
        if (activeTab.value === 'engine') {
          refreshEngineStatus();
        }
      }
    }
  );

  // 监听标签切换
  watch(activeTab, (newTab) => {
    if (newTab === 'engine' && visible.value) {
      refreshEngineStatus();
    }
  });

  watch(visible, (newVal) => {
    emit('update:show', newVal);
  });

  watch(() => props.settings, initSettings, { deep: true });

  // 暴露方法给父组件
  const openTab = (tabName) => {
    activeTab.value = tabName;
    visible.value = true;
  };

  defineExpose({
    openTab,
  });
</script>

<style scoped>
  /* 抽屉基础样式 */
  .advanced-drawer :deep(.ant-drawer-body) {
    padding: 0;
    background: var(--editor-inset-bg, #f8fafc);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .drawer-content {
    display: flex;
    min-height: 400px;
    height: 100%;
    width: 100%;
  }

  /* Tab 样式优化 */
  .advanced-drawer .ant-tabs {
    flex: 1;
    min-width: 0;
    display: flex;
  }

  .advanced-drawer :deep(.ant-tabs-nav) {
    background: var(--editor-panel-bg, #fff);
    border-right: 1px solid var(--editor-border, #e2e8f0);
    padding: 12px 0;
    flex-shrink: 0;
    height: 100%;
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

  .advanced-drawer :deep(.ant-tabs-tab) {
    padding: 12px 20px !important;
    margin: 4px 8px;
    border-radius: 8px;
    transition: all 0.2s ease;
    font-size: 14px;
    color: var(--editor-muted, #64748b);
  }

  .advanced-drawer :deep(.ant-tabs-tab:hover) {
    background: var(--editor-hover-bg, #f1f5f9);
    color: var(--editor-primary, #334155);
  }

  .advanced-drawer :deep(.ant-tabs-tab-active) {
    background: linear-gradient(135deg, #3e7bfa 0%, #2f63d8 100%);
    color: #fff !important;
    font-weight: 500;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  }

  .advanced-drawer :deep(.ant-tabs-ink-bar) {
    display: none;
  }

  .tab-content-scroll {
    height: 100%;
    overflow-y: scroll;
  }

  .tab-content {
    padding: 24px;
    width: 100%;
    box-sizing: border-box;
  }

  /* 设置卡片 - 现代风格 */
  .settings-card {
    margin-bottom: 20px;
    border-radius: 12px;
    border: 1px solid var(--editor-border, #e2e8f0);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
    transition: all 0.3s ease;
    background: var(--editor-panel-bg, #fff);
    overflow: hidden;
  }

  .settings-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    border-color: #cbd5e1;
  }

  .settings-card:last-child {
    margin-bottom: 0;
  }

  .settings-card :deep(.ant-card-head) {
    padding: 16px 20px;
    background: var(--editor-inset-bg, #f8fafc);
    border-bottom: 1px solid var(--editor-border, #e2e8f0);
    min-height: auto;
  }

  .settings-card :deep(.ant-card-head-title) {
    font-size: 15px;
    font-weight: 600;
    color: var(--editor-primary, #1e293b);
  }

  .settings-card :deep(.ant-card-body) {
    padding: 0;
  }

  /* 设置项 - 现代卡片风格 */
  .setting-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--editor-border, #f1f5f9);
    transition: all 0.2s ease;
    gap: 20px;
  }

  .setting-item:hover {
    background: var(--editor-inset-bg, #fafbfc);
  }

  .setting-item:last-child {
    border-bottom: none;
  }

  .setting-label {
    flex: 1;
    min-width: 0;
  }

  .setting-label > span:first-child {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: var(--editor-primary, #1e293b);
    margin-bottom: 4px;
    line-height: 1.4;
  }

  .setting-description {
    display: block;
    font-size: 12px;
    color: var(--editor-muted, #94a3b8);
    line-height: 1.5;
  }

  .setting-control {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    gap: 8px;
  }

  /* 底部按钮区域 */
  .drawer-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 12px 24px;
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
    margin: 0 -20px;
    padding: 12px 20px;
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
    margin-top: 16px;
    padding: 16px;
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

  /* 响应式调整 */
  @media (max-width: 768px) {
    .tab-content {
      padding: 16px;
    }

    .setting-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .setting-control {
      width: 100%;
      justify-content: flex-end;
    }
  }

</style>
