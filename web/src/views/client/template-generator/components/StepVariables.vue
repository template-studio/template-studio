<template>
  <div class="step-variables">
    <div class="variables-content">
      <!-- 加载中 -->
      <div v-if="loading" class="loading-container">
        <n-spin size="large">
          <template #description>正在加载模板变量…</template>
        </n-spin>
      </div>

      <!-- 表单 -->
      <div v-else class="variables-form">
        <div class="form-header">
          <div class="form-title-section">
            <h2 class="form-title">配置项目变量</h2>
            <p class="form-desc">请填写以下信息来配置您的项目</p>
          </div>
          <div class="mode-tabs">
            <n-tabs v-model:value="currentMode" type="segment" size="small">
              <n-tab-pane name="normal" tab="普通模式" />
              <n-tab-pane name="advanced" tab="高级模式" />
            </n-tabs>
          </div>
        </div>

        <!-- 普通模式 -->
        <div v-if="currentMode === 'normal'" class="normal-mode">
          <div v-if="customVariables.length" class="variable-section">
            <h3 class="section-title">
              <n-icon><CodeSlash /></n-icon>
              自定义变量
              <span class="section-subtitle">({{ customVariables.length }})</span>
            </h3>

            <div class="variable-grid">
              <div
                v-for="v in customVariables"
                :key="v.name"
                class="variable-item"
                :class="{ full: v.type === 'object' || v.type === 'object_arr' }"
              >
                <label>
                  <span class="name">{{ v.name }}</span>
                  <span v-if="v.desc" class="desc">- {{ v.desc }}</span>
                  <span v-if="v.required" class="required">*</span>
                  <span class="badge">{{ v.type }}</span>
                </label>

                <!-- 字符串 -->
                <n-input
                  v-if="v.type === 'string'"
                  v-model:value="formData[v.name]"
                  :placeholder="v.desc || '请输入字符串'"
                />

                <!-- 数字 -->
                <n-input-number
                  v-else-if="v.type === 'number'"
                  v-model:value="formData[v.name]"
                  :placeholder="v.desc || '请输入数字'"
                  style="width: 100%"
                />

                <!-- 布尔 -->
                <n-switch v-else-if="v.type === 'boolean'" v-model:value="formData[v.name]" />

                <!-- 列表 -->
                <div v-else-if="v.type === 'list'" class="list-box">
                  <n-dynamic-tags
                    :key="`list-${v.name}`"
                    v-model:value="formData[v.name]"
                    :placeholder="v.desc || '按回车添加'"
                    :max="10"
                    @create="(label) => createTag(v.name, label)"
                  />
                </div>

                <!-- 对象 / 对象数组 -->
                <n-input
                  v-else
                  v-model:value="formData[v.name]"
                  type="textarea"
                  :placeholder="v.desc || '请输入 JSON'"
                  :autosize="{ minRows: 3, maxRows: 6 }"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- 高级模式 -->
        <div v-else-if="currentMode === 'advanced'" class="advanced-mode">
          <div class="editor-wrap">
            <div class="editor-header">
              <span>JSON 编辑器</span>
              <div class="actions">
                <n-button size="small" quaternary @click="formatJson">格式化</n-button>
                <n-button size="small" type="info" quaternary @click="syncFromNormal"
                  >同步普通模式</n-button
                >
              </div>
            </div>
            <div ref="jsonEditorEl" class="json-editor"></div>
            <div class="editor-footer">
              <span v-if="jsonOk" class="ok">✅ JSON 格式正确</span>
              <span v-else class="err">❌ {{ jsonErr }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部 -->
    <div class="step-actions">
      <n-button @click="$emit('prev')">上一步</n-button>
      <n-button type="primary" :disabled="!valid" @click="handleNext">下一步</n-button>
    </div>
  </div>
</template>

<script setup>
  import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
  import {
    NSpin,
    NTabs,
    NTabPane,
    NInput,
    NInputNumber,
    NSwitch,
    NDynamicTags,
    NButton,
    NIcon,
  } from 'naive-ui';
  import { CodeSlash } from '@vicons/ionicons5';
  import { EditorView } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { json } from '@codemirror/lang-json';
  import { dracula } from '@uiw/codemirror-theme-dracula';
  import { getTemplateVariables } from '@/api/templateFiles';

  /* props & emit */
  const props = defineProps({
    templateInfo: { type: Object, default: null },
    selectedVersion: { type: String, default: '' },
    variables: { type: Object, default: () => ({}) },
  });
  const emit = defineEmits(['prev', 'next', 'update-variables']);

  /* 状态 */
  const loading = ref(false);
  const currentMode = ref('normal');
  const jsonEditorEl = ref(null);
  let cmEditor = null;
  const jsonOk = ref(true);
  const jsonErr = ref('');

  /* 变量 & 表单 */
  const customVariables = ref([]); // { name, type, desc, required, default }
  const formData = ref({});

  /* 加载变量定义 */
  async function loadVars() {
    if (!props.templateInfo?.id) return;
    loading.value = true;
    try {
      // 使用新的 API，根据版本加载变量定义
      const { data: res } = await getTemplateVariables({
        templateId: props.templateInfo.id,
        version: props.selectedVersion || undefined,
      });
      const schema = JSON.parse(res.data?.fieldSchemaJson || '{}');
      customVariables.value = Object.entries(schema).map(([k, v]) => ({
        name: k,
        type: v.type || 'string',
        desc: v.description || '',
        required: !!v.required,
        default: v.default,
      }));
      initForm();
    } finally {
      loading.value = false;
    }
  }

  /* 初始化/重置表单 */
  function initForm() {
    const base = { ...props.variables };
    customVariables.value.forEach((v) => {
      if (base[v.name] === undefined) {
        base[v.name] = v.default ?? defaultByType(v.type);
      }
    });
    formData.value = base;
  }

  function defaultByType(t) {
    switch (t) {
      case 'boolean':
        return false;
      case 'number':
        return 0;
      case 'list':
        return [];
      case 'object':
        return '{}';
      case 'object_arr':
        return '[]';
      default:
        return '';
    }
  }

  /* 校验 */
  const valid = computed(() => {
    if (currentMode.value === 'advanced') return jsonOk.value;
    for (const v of customVariables.value) {
      if (v.required && !formData.value[v.name]) return false;
      if ((v.type === 'object' || v.type === 'object_arr') && formData.value[v.name]) {
        try {
          JSON.parse(formData.value[v.name]);
        } catch {
          return false;
        }
      }
    }
    return true;
  });

  /* 普通 ↔ 高级 数据同步 */
  watch(currentMode, async (newMode, oldMode) => {
    if (newMode === 'advanced') {
      await nextTick();
      if (!jsonEditorEl.value) return;

      // 如果编辑器已存在，先销毁
      if (cmEditor) {
        cmEditor.destroy();
        cmEditor = null;
      }

      // 重新初始化编辑器并同步数据
      initCodeMirror();

      // 确保数据同步到编辑器
      if (cmEditor && formData.value) {
        try {
          const currentContent = cmEditor.state.doc.toString();
          const newData = JSON.stringify(formData.value, null, 2);
          if (currentContent !== newData) {
            cmEditor.dispatch({
              changes: { from: 0, to: cmEditor.state.doc.length, insert: newData },
            });
          }
        } catch (e) {
          console.warn('Failed to sync data to editor:', e);
        }
      }
    } else if (oldMode === 'advanced') {
      // 从高级模式切换到普通模式时，同步数据到 formData
      if (cmEditor) {
        try {
          const data = JSON.parse(cmEditor.state.doc.toString());
          Object.assign(formData.value, data);
        } catch (e) {
          console.warn('Failed to parse JSON from editor:', e);
        }
      }
    }
  });

  function initCodeMirror() {
    if (!jsonEditorEl.value) return;

    const state = EditorState.create({
      doc: JSON.stringify(formData.value, null, 2),
      extensions: [dracula, json(), EditorView.updateListener.of(() => validateJson())],
    });
    cmEditor = new EditorView({ state, parent: jsonEditorEl.value });
    validateJson();
  }

  function validateJson() {
    try {
      JSON.parse(cmEditor.state.doc.toString());
      jsonOk.value = true;
      jsonErr.value = '';
    } catch (e) {
      jsonOk.value = false;
      jsonErr.value = e.message;
    }
  }

  function formatJson() {
    if (!cmEditor) return;
    try {
      const doc = JSON.parse(cmEditor.state.doc.toString());
      cmEditor.dispatch({
        changes: { from: 0, to: cmEditor.state.doc.length, insert: JSON.stringify(doc, null, 2) },
      });
    } catch {}
  }

  function syncFromNormal() {
    if (!cmEditor) return;
    cmEditor.dispatch({
      changes: {
        from: 0,
        to: cmEditor.state.doc.length,
        insert: JSON.stringify(formData.value, null, 2),
      },
    });
  }

  /* 处理标签创建 */
  function createTag(variableName, label) {
    if (!formData.value[variableName]) {
      formData.value[variableName] = [];
    }
    if (!formData.value[variableName].includes(label)) {
      formData.value[variableName].push(label);
    }
    return false; // 阻止默认行为
  }

  /* 下一步 */
  function handleNext() {
    let out = {};
    if (currentMode.value === 'normal') {
      out = { ...formData.value };
      customVariables.value
        .filter((v) => v.type === 'object' || v.type === 'object_arr')
        .forEach((v) => {
          if (out[v.name]) {
            try {
              out[v.name] = JSON.parse(out[v.name]);
            } catch {}
          }
        });
    } else {
      out = JSON.parse(cmEditor.state.doc.toString());
    }
    emit('update-variables', out);
    emit('next');
  }

  /* 监听外部 variables 变化（仅第一次） */
  let inited = false;
  watch(
    () => props.variables,
    (v) => {
      if (!inited && v && Object.keys(v).length) {
        Object.assign(formData.value, v);
        inited = true;
      }
    },
    { immediate: true }
  );

  /* 监听模板变化 */
  watch(
    () => props.templateInfo,
    (t) => t?.id && loadVars(),
    { immediate: true }
  );

  /* 监听版本变化，重新加载变量定义 */
  watch(
    () => props.selectedVersion,
    () => {
      if (props.templateInfo?.id) {
        loadVars();
      }
    }
  );

  /* 表单变化自动向上同步 */
  watch(formData, () => emit('update-variables', formData.value), { deep: true });

  /* 组件卸载时清理编辑器 */
  onUnmounted(() => {
    if (cmEditor) {
      cmEditor.destroy();
      cmEditor = null;
    }
  });
</script>

<style scoped>
  .step-variables {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .variables-content {
    flex: 1;
    padding: 32px;
    overflow: auto;
  }
  .loading-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .variables-form {
    max-width: 800px;
    margin: 0 auto;
  }
  .form-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;
    gap: 16px;
  }
  .form-title-section {
    flex: 1;
  }
  .mode-tabs {
    flex-shrink: 0;
  }
  .form-title {
    font-size: 24px;
    font-weight: 600;
    color: #0f172a;
    margin-bottom: 4px;
  }
  .form-desc {
    color: #64748b;
    font-size: 14px;
  }
  .mode-tabs :deep(.n-tabs-nav) {
    background: #f1f5f9;
    border-radius: 6px;
    padding: 4px;
  }
  .mode-tabs :deep(.n-tabs-tab) {
    border-radius: 4px;
    margin: 0 2px;
    min-width: 80px;
    justify-content: center;
  }
  .mode-tabs :deep(.n-tabs-tab:not(.n-tabs-tab--active)) {
    background: transparent;
    color: #64748b;
  }
  .mode-tabs :deep(.n-tabs-tab.n-tabs-tab--active) {
    background: #fff;
    color: #22c55e;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  }
  .mode-tabs :deep(.n-tabs-tab:hover) {
    background: rgba(255, 255, 255, 0.8);
  }
  .variable-section {
    margin-bottom: 24px;
  }
  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 16px;
    font-weight: 600;
    color: #0f172a;
    margin-bottom: 12px;
    border-bottom: 1px solid #f1f5f9;
    padding-bottom: 8px;
  }
  .section-subtitle {
    font-size: 13px;
    color: #94a3b8;
    margin-left: 6px;
    font-weight: 400;
  }
  .variable-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
  }
  .variable-item.full {
    grid-column: 1/-1;
  }
  .variable-item label {
    font-size: 14px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 4px;
  }
  .name {
    font-family: monospace;
  }
  .desc {
    font-weight: 400;
    color: #94a3b8;
    font-size: 12px;
  }
  .required {
    color: #d03050;
    margin-left: 2px;
  }
  .badge {
    font-size: 11px;
    background: #e6f7ff;
    color: #1890ff;
    padding: 2px 6px;
    border-radius: 3px;
    margin-left: auto;
  }
  .list-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .editor-wrap {
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    overflow: hidden;
  }
  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #f8fafc;
    border-bottom: 1px solid #e2e8f0;
    font-size: 14px;
    font-weight: 500;
    color: #334155;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  .json-editor {
    height: 400px;
  }
  .editor-footer {
    padding: 8px 16px;
    background: #f8fafc;
    border-top: 1px solid #e2e8f0;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
    color: #64748b;
  }
  .ok {
    color: #22c55e;
  }
  .err {
    color: #d03050;
  }
  .step-actions {
    padding: 16px 24px;
    border-top: 1px solid #e2e8f0;
    background: #fff;
    display: flex;
    justify-content: space-between;
  }
</style>
