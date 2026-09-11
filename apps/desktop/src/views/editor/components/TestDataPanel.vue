<template>
  <!-- 测试数据工作室层(#200 补10):占满内容区,左表单/右 JSON 双向同步。
       由父组件在 edit-main 内绝对定位;v-show 保活,切换顺滑 -->
  <div v-show="visible" class="td-studio">
    <!-- 头部 -->
    <div class="td-head">
      <span class="td-title">测试数据</span>
      <span class="td-sub">表单与 JSON 双向同步,保存后预览自动重渲</span>
      <div class="td-actions">
        <button
          v-for="f in ['json', 'yaml']"
          :key="f"
          class="td-act fmt"
          :class="{ on: dataFormat === f }"
          @click="dataFormat = f"
        >{{ f.toUpperCase() }}</button>
        <button class="td-act" title="重新生成(保留已有字段的数据)" @click="handleRegenerate">
          <RedoOutlined :spin="loading" />
        </button>
        <button class="td-act" title="复制" @click="handleCopy"><CopyOutlined /></button>
        <a-button size="small" type="primary" :loading="saving" @click="handleSave">
          <template #icon><CheckOutlined /></template>保存并应用
        </a-button>
        <button class="td-act" title="关闭" @click="emit('update:show', false)"><CloseOutlined /></button>
      </div>
    </div>

    <!-- 主体:左表单 | 分栏拖拽 | 右代码 -->
    <div class="td-main">
      <div class="td-form" :style="{ width: leftPct + '%' }">
        <div v-if="hasSchema" class="td-form-inner">
          <FieldGroup :schema="serverSchema" :data="testData" />
        </div>
        <div v-else class="td-empty">
          暂无变量定义,请先在「变量设计器」中设计变量,
          或直接在右侧编辑 JSON 后保存。
        </div>
      </div>

      <div class="td-split" @mousedown="startSplit"><span class="td-split-bar"></span></div>

      <div class="td-json">
        <div ref="editorRef" class="td-editor"></div>
        <div v-if="parseError" class="td-parse-err">{{ parseError }}</div>
      </div>
    </div>
  </div>
</template>

<script setup>
// 测试数据工作室(#200 补10):单数据源 testData + 双向同步——
// 表单直接改写 testData(深度监听→防抖序列化到右侧编辑器);
// 右侧编辑器解析成功→整体替换 testData(表单响应式刷新),来自编辑器的回写用标志位阻断环路。
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { message } from 'ant-design-vue';
import { RedoOutlined, CopyOutlined, CheckOutlined, CloseOutlined } from '@ant-design/icons-vue';
import { getTemplateTestData, setTemplateTestData } from '@/api/editor/templateExpose';
import { useSchemaStore } from './QuickDesignDrawer/composables/useSchemaStore';
import { useThemeStore } from '@/stores/theme';
import { dracula } from '@uiw/codemirror-theme-dracula';
import FieldGroup from './FieldGroup.vue';
import { EditorView, basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';
import { json } from '@codemirror/lang-json';
import { yaml } from '@codemirror/lang-yaml';
import * as YAML from 'js-yaml';

const props = defineProps({
  show: { type: Boolean, default: false },
  templateId: { type: [String, Number], required: true },
});
const emit = defineEmits(['update:show', 'updated']);

const visible = computed(() => props.show);
const editorRef = ref(null);
const testData = ref({});
const dataFormat = ref('json');
const loading = ref(false);
const saving = ref(false);
const parseError = ref('');
const leftPct = ref(50);
let editorView = null;
let syncTimer = null;
let fromEditor = false;

const { serverSchema } = useSchemaStore(props.templateId);
const hasSchema = computed(() => Object.keys(serverSchema.value || {}).length > 0);
// 行号/编辑区主题适配:暗色挂 dracula(与主编辑器一致),亮色用默认
const themeStore = useThemeStore();

// 按 schema 生成测试数据(已有字段优先保留),与 TestDataModal 同源逻辑
const generateTestData = (schema, existingData = {}) => {
  const data = {};
  if (!schema || typeof schema !== 'object') return data;

  Object.entries(schema).forEach(([key, variable]) => {
    if (!variable || typeof variable !== 'object') return;

    if (existingData && Object.prototype.hasOwnProperty.call(existingData, key)) {
      switch (variable.type) {
        case 'object':
          if (variable.properties && typeof existingData[key] === 'object' && existingData[key] !== null) {
            data[key] = generateTestData(variable.properties, existingData[key]);
          } else {
            data[key] = existingData[key];
          }
          break;
        case 'object_arr':
          if (Array.isArray(existingData[key])) {
            data[key] = existingData[key];
          } else if (variable.items && variable.items.properties) {
            const itemData = generateTestData(variable.items.properties);
            data[key] = [itemData, { ...itemData }];
          } else {
            data[key] = [];
          }
          break;
        default:
          data[key] = existingData[key];
      }
      return;
    }

    switch (variable.type) {
      case 'string': data[key] = variable.default || `示例${key}`; break;
      case 'integer': data[key] = variable.default || 42; break;
      case 'number': data[key] = variable.default || 3.14; break;
      case 'boolean': data[key] = variable.default !== undefined ? variable.default : true; break;
      case 'array': data[key] = variable.default || ['item1', 'item2']; break;
      case 'object':
        data[key] = variable.properties ? generateTestData(variable.properties) : {};
        break;
      case 'object_arr':
        if (variable.items && variable.items.properties) {
          const itemData = generateTestData(variable.items.properties);
          data[key] = [itemData, { ...itemData }];
        } else {
          data[key] = [];
        }
        break;
      case 'enum':
        data[key] = variable.enum && variable.enum.length > 0 ? variable.enum[0] : variable.default || '';
        break;
      case 'secret': data[key] = variable.default || '***保密信息***'; break;
      default: data[key] = variable.default || '';
    }
  });
  return data;
};

const loadTestData = async () => {
  loading.value = true;
  try {
    const result = await getTemplateTestData({ templateId: props.templateId });
    testData.value = generateTestData(serverSchema.value, result || {});
  } catch (error) {
    console.error('加载测试数据失败:', error);
    testData.value = generateTestData(serverSchema.value);
  } finally {
    loading.value = false;
  }
};

const formatDataToString = (data) =>
  dataFormat.value === 'json'
    ? JSON.stringify(data, null, 2)
    : YAML.dump(data, { indent: 2, lineWidth: -1 });

const parseContent = (content) =>
  dataFormat.value === 'json' ? JSON.parse(content) : YAML.load(content);

// ---------- 右侧编辑器 ----------

const initEditor = () => {
  if (!editorRef.value) return;
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
  const extensions = [
    basicSetup,
    ...(themeStore.isDark ? [dracula] : []),
    EditorView.theme({
      '&': { fontSize: '12.5px', fontFamily: 'Monaco, Menlo, Consolas, monospace', height: '100%' },
      '.cm-scroller': { overflow: 'auto' },
      '.cm-content': { padding: '10px' },
    }),
    EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      try {
        const parsed = parseContent(update.state.doc.toString());
        parseError.value = '';
        // 环路阻断:编辑器侧发起的替换不回写编辑器
        fromEditor = true;
        testData.value = parsed || {};
      } catch (error) {
        parseError.value = `${dataFormat.value.toUpperCase()} 解析错误: ${error.message}`;
      }
    }),
  ];
  extensions.push(dataFormat.value === 'yaml' ? yaml() : json());

  editorView = new EditorView({
    state: EditorState.create({ doc: formatDataToString(testData.value), extensions }),
    parent: editorRef.value,
  });
};

// ---------- 表单 → 编辑器同步(深度监听 + 防抖) ----------
watch(
  testData,
  () => {
    if (fromEditor) {
      fromEditor = false;
      return;
    }
    if (!editorView) return;
    clearTimeout(syncTimer);
    syncTimer = setTimeout(() => {
      if (!editorView) return;
      const next = formatDataToString(testData.value);
      if (editorView.state.doc.toString() !== next) {
        editorView.dispatch({
          changes: { from: 0, to: editorView.state.doc.length, insert: next },
        });
      }
    }, 250);
  },
  { deep: true }
);

// 格式切换:重挂编辑器(以当前数据为新文档)
watch(dataFormat, () => {
  parseError.value = '';
  initEditor();
});

// 明暗主题切换:重挂编辑器
watch(() => themeStore.isDark, () => {
  initEditor();
});

// ---------- 头部动作 ----------

const handleRegenerate = () => {
  testData.value = generateTestData(serverSchema.value, testData.value);
  message.success('测试数据已重新生成');
};

const handleCopy = async () => {
  try {
    await navigator.clipboard.writeText(formatDataToString(testData.value));
    message.success(`已复制(${dataFormat.value.toUpperCase()}格式)`);
  } catch {
    message.error('复制失败');
  }
};

const handleSave = async () => {
  // 以编辑器内容为准(允许只改 JSON 未触碰表单的场景)
  let toSave;
  try {
    toSave = editorView ? parseContent(editorView.state.doc.toString()) : testData.value;
    parseError.value = '';
  } catch (error) {
    parseError.value = `${dataFormat.value.toUpperCase()} 解析错误: ${error.message}`;
    message.error('内容格式有误,请先修正后再保存');
    return;
  }
  saving.value = true;
  try {
    await setTemplateTestData({ templateId: props.templateId, testData: toSave || {} });
    emit('updated', toSave || {});
    message.success('测试数据已保存并应用');
  } catch (error) {
    console.error('保存测试数据失败:', error);
    message.error('保存失败: ' + (error.message || error));
  } finally {
    saving.value = false;
  }
};

// ---------- 分栏拖拽 ----------

let splitStartX = 0;
let splitStartPct = 50;
const startSplit = (e) => {
  e.preventDefault();
  splitStartX = e.clientX;
  splitStartPct = leftPct.value;
  const container = e.target.closest('.td-main');
  const totalW = container ? container.clientWidth : 1;
  const onMove = (ev) => {
    const deltaPct = ((ev.clientX - splitStartX) / totalW) * 100;
    leftPct.value = Math.min(75, Math.max(25, splitStartPct + deltaPct));
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';
};

onMounted(async () => {
  await loadTestData();
  initEditor();
});

onUnmounted(() => {
  clearTimeout(syncTimer);
  if (editorView) {
    editorView.destroy();
    editorView = null;
  }
});
</script>

<style scoped>
/* 不设 width:由父级 .designer-layer 的 left/right 拉伸(设 width:100% 会与 left 过约束致右缘溢出) */
.td-studio {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--editor-panel-bg, #fff);
}

.td-head {
  height: 42px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 12px;
  border-bottom: 1px solid var(--editor-border, #e2e8f0);
  background: var(--editor-inset-bg, #fafbfc);
}

.td-title { font-size: 13px; font-weight: 700; color: var(--color-text, #1b1c1f); white-space: nowrap; }
.td-sub { flex: 1; min-width: 0; font-size: 11px; color: var(--editor-muted, #94a3b8); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.td-actions { display: flex; align-items: center; gap: 6px; }

.td-act {
  min-width: 26px; height: 26px; border: none; background: transparent; border-radius: 5px;
  cursor: pointer; color: var(--color-text-secondary, #64748b); font-size: 13px;
  display: inline-flex; align-items: center; justify-content: center;
}
.td-act:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.td-act.fmt { font-size: 10px; font-weight: 600; padding: 0 6px; }
.td-act.fmt.on { background: var(--color-nav-active, #eef0ec); color: var(--color-text, #1b1c1f); }

.td-main { flex: 1; min-height: 0; display: flex; }

.td-form { min-width: 0; overflow-y: auto; }
.td-form-inner { padding: 14px 18px 24px; max-width: 720px; }

.td-empty { padding: 40px 24px; font-size: 12.5px; color: var(--editor-muted, #94a3b8); text-align: center; }

.td-split {
  flex-shrink: 0;
  width: 9px;
  margin: 0 -4px;
  z-index: 5;
  cursor: col-resize;
  display: flex;
  align-items: center;
  justify-content: center;
}
.td-split-bar { width: 2px; height: 36px; border-radius: 2px; background: var(--editor-border, #d8dee6); transition: background 0.15s; }
.td-split:hover .td-split-bar { background: var(--editor-accent, #16a34a); }

.td-json { flex: 1; min-width: 0; position: relative; border-left: 1px solid var(--editor-border, #e2e8f0); display: flex; }

.td-editor { flex: 1; min-width: 0; }
.td-editor :deep(.cm-editor) { height: 100%; }

.td-parse-err {
  position: absolute; left: 8px; right: 8px; bottom: 8px;
  padding: 6px 10px; border-radius: 6px;
  background: rgba(220, 38, 38, 0.92); color: #fff;
  font-size: 11.5px; font-family: Consolas, monospace;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}
</style>
