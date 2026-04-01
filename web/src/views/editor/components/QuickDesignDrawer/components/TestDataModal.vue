<template>
  <n-modal v-model:show="showModal" :mask-closable="false">
    <n-card
      style="width: 900px; max-height: 700px"
      title="生成测试数据"
      :bordered="false"
      size="huge"
    >
      <template #header-extra>
        <n-button quaternary circle @click="handleClose">
          <template #icon>
            <n-icon><CloseOutline /></n-icon>
          </template>
        </n-button>
      </template>

      <n-space vertical>
        <div style="margin-bottom: 12px">
          <n-text depth="3">
            基于当前变量定义自动生成的测试数据，你可以直接编辑这些数据用于测试模板。
          </n-text>
        </div>

        <div
          style="
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 12px;
          "
        >
          <!-- 格式切换 -->
          <n-button-group size="small">
            <n-button
              :type="dataFormat === 'json' ? 'primary' : 'default'"
              @click="dataFormat = 'json'"
            >
              JSON
            </n-button>
            <n-button
              :type="dataFormat === 'yaml' ? 'primary' : 'default'"
              @click="dataFormat = 'yaml'"
            >
              YAML
            </n-button>
          </n-button-group>

          <!-- 操作按钮 -->
          <div style="display: flex; gap: 8px">
            <n-button size="small" @click="handleRegenerate" quaternary>
              <template #icon>
                <n-icon><RefreshOutline /></n-icon>
              </template>
              重新生成
            </n-button>
            <n-button size="small" @click="handleCopy" quaternary>
              <template #icon>
                <n-icon><CopyOutline /></n-icon>
              </template>
              复制数据
            </n-button>
            <n-button size="small" @click="handleSave" type="primary">
              <template #icon>
                <n-icon><SaveOutline /></n-icon>
              </template>
              保存数据
            </n-button>
          </div>
        </div>

        <div ref="editorRef" class="test-data-editor"></div>
      </n-space>

      <template #footer>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <n-text depth="3" style="font-size: 12px">
            数据将保存到服务器，模板ID：{{ templateId }}
          </n-text>
          <div style="display: flex; gap: 12px">
            <n-button @click="handleSave" type="primary">
              <template #icon>
                <n-icon><SaveOutline /></n-icon>
              </template>
              保存并应用
            </n-button>
            <n-button @click="handleClose">关闭</n-button>
          </div>
        </div>
      </template>
    </n-card>
  </n-modal>
</template>

<script setup>
  import { ref, watch, nextTick, onUnmounted } from 'vue';
  import { NModal, NCard, NButton, NButtonGroup, NSpace, NText, NIcon, useMessage } from 'naive-ui';
  import { CloseOutline, RefreshOutline, CopyOutline, SaveOutline } from '@vicons/ionicons5';
  import { getTemplateTestData, setTemplateTestData } from '@/api/templateExpose';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { json } from '@codemirror/lang-json';
  import { yaml } from '@codemirror/lang-yaml';
  import * as YAML from 'js-yaml';

  /**
   * TestDataModal 组件
   * 测试数据生成和编辑模态框
   */

  // Props
  const props = defineProps({
    show: {
      type: Boolean,
      default: false,
    },
    templateId: {
      type: [String, Number],
      required: true,
    },
    schema: {
      type: Object,
      default: () => ({}),
    },
  });

  // Emits
  const emit = defineEmits(['update:show', 'test-data-updated']);

  const message = useMessage();

  // Refs
  const showModal = ref(false);
  const editorRef = ref(null);
  const testData = ref({});
  const dataFormat = ref('json'); // 'json' | 'yaml'
  let editorView = null;

  // ========== 方法 ==========

  /**
   * 生成测试数据
   */
  const generateTestData = (schema, existingData = {}) => {
    const data = {};

    if (!schema || typeof schema !== 'object') {
      return data;
    }

    Object.entries(schema).forEach(([key, variable]) => {
      if (!variable || typeof variable !== 'object') return;

      // 如果现有数据中已有该字段，优先使用现有数据
      if (existingData && existingData.hasOwnProperty(key)) {
        switch (variable.type) {
          case 'object':
            if (
              variable.properties &&
              typeof existingData[key] === 'object' &&
              existingData[key] !== null
            ) {
              data[key] = generateTestData(variable.properties, existingData[key]);
            } else {
              data[key] = existingData[key];
            }
            break;
          case 'object_arr':
            if (Array.isArray(existingData[key])) {
              data[key] = existingData[key];
            } else {
              if (variable.items && variable.items.properties) {
                const itemData = generateTestData(variable.items.properties);
                data[key] = [itemData, { ...itemData }];
              } else {
                data[key] = [];
              }
            }
            break;
          default:
            data[key] = existingData[key];
        }
        return;
      }

      // 生成新的默认值
      switch (variable.type) {
        case 'string':
          data[key] = variable.default || `示例${key}`;
          break;
        case 'integer':
          data[key] = variable.default || 42;
          break;
        case 'number':
          data[key] = variable.default || 3.14;
          break;
        case 'boolean':
          data[key] = variable.default !== undefined ? variable.default : true;
          break;
        case 'array':
          data[key] = variable.default || ['item1', 'item2'];
          break;
        case 'object':
          if (variable.properties) {
            data[key] = generateTestData(variable.properties);
          } else {
            data[key] = {};
          }
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
          data[key] =
            variable.enum && variable.enum.length > 0 ? variable.enum[0] : variable.default || '';
          break;
        case 'secret':
          data[key] = variable.default || '***保密信息***';
          break;
        default:
          data[key] = variable.default || '';
      }
    });

    return data;
  };

  /**
   * 从服务器加载测试数据
   */
  const loadTestData = async () => {
    try {
      const result = await getTemplateTestData({ templateId: props.templateId });
      if (result && Object.keys(result).length > 0) {
        testData.value = result;
      } else {
        testData.value = generateTestData(props.schema);
      }
    } catch (error) {
      console.error('加载测试数据失败:', error);
      testData.value = generateTestData(props.schema);
    }
  };

  /**
   * 初始化编辑器
   */
  const initEditor = () => {
    if (!editorRef.value) return;

    // 清除现有编辑器
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }

    // 根据格式选择语言包
    const extensions = [
      basicSetup,
      EditorView.theme({
        '&': {
          fontSize: '14px',
          fontFamily: 'Monaco, Menlo, "Ubuntu Mono", Consolas, monospace',
        },
        '.cm-content': {
          padding: '12px',
          minHeight: '200px',
        },
        '.cm-editor': {
          borderRadius: '6px',
          border: '1px solid #e0e0e6',
        },
        '.cm-focused': {
          outline: 'none',
          borderColor: '#18a058',
        },
      }),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          try {
            const content = update.state.doc.toString();
            if (dataFormat.value === 'json') {
              const newData = JSON.parse(content);
              testData.value = newData;
            } else {
              // YAML 格式，使用 js-yaml 解析
              const newData = YAML.load(content);
              testData.value = newData;
            }
          } catch (error) {
            // 格式错误，暂不处理
          }
        }
      }),
    ];

    // 根据格式添加语言包
    if (dataFormat.value === 'yaml') {
      extensions.push(yaml());
    } else {
      extensions.push(json());
    }

    const state = EditorState.create({
      doc: formatDataToString(),
      extensions,
    });

    editorView = new EditorView({
      state,
      parent: editorRef.value,
    });
  };

  /**
   * 将数据格式化为字符串
   */
  const formatDataToString = () => {
    if (dataFormat.value === 'json') {
      return JSON.stringify(testData.value, null, 2);
    } else {
      // YAML 格式
      return YAML.dump(testData.value, {
        indent: 2,
        lineWidth: -1,
      });
    }
  };

  /**
   * 重新生成测试数据
   */
  const handleRegenerate = () => {
    let existingData = {};

    // 尝试获取当前编辑器中的数据作为现有数据
    if (editorView) {
      try {
        const editorContent = editorView.state.doc.toString();
        if (dataFormat.value === 'json') {
          existingData = JSON.parse(editorContent);
        } else {
          existingData = YAML.load(editorContent);
        }
      } catch (e) {
        console.warn('解析当前编辑器数据失败，将使用空数据重新生成:', e);
        existingData = {};
      }
    }

    // 重新生成时保留现有字段的数据
    testData.value = generateTestData(props.schema, existingData);

    if (editorView) {
      const transaction = editorView.state.update({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: formatDataToString(),
        },
      });
      editorView.dispatch(transaction);
    }

    message.success('测试数据已重新生成');
  };

  /**
   * 复制测试数据
   */
  const handleCopy = async () => {
    try {
      const dataStr = formatDataToString();
      await navigator.clipboard.writeText(dataStr);
      message.success(`测试数据已复制到剪贴板 (${dataFormat.value.toUpperCase()}格式)`);
    } catch (error) {
      message.error('复制失败');
    }
  };

  /**
   * 保存测试数据
   */
  const handleSave = async () => {
    try {
      // 从编辑器获取最新数据
      if (editorView) {
        const content = editorView.state.doc.toString();
        const parsed = JSON.parse(content);
        testData.value = parsed;
      }

      await setTemplateTestData({
        templateId: props.templateId,
        testData: testData.value,
      });

      // 通知父组件
      emit('test-data-updated', testData.value);
      message.success('测试数据已保存');
    } catch (error) {
      console.error('保存测试数据失败:', error);
      message.error('保存失败: ' + error.message);
    }
  };

  /**
   * 关闭模态框
   */
  const handleClose = () => {
    emit('update:show', false);
  };

  // ========== 监听 ==========

  watch(
    () => props.show,
    async (newVal) => {
      showModal.value = newVal;
      if (newVal) {
        await loadTestData();
        nextTick(() => {
          initEditor();
        });
      }
    }
  );

  watch(showModal, (newVal) => {
    if (!newVal) {
      emit('update:show', false);
    }
  });

  // 监听格式变化，重新初始化编辑器
  watch(dataFormat, () => {
    if (editorView) {
      initEditor();
    }
  });

  // ========== 生命周期 ==========

  onUnmounted(() => {
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
  });
</script>

<style scoped>
  .test-data-editor {
    font-size: 13px;
  }
</style>
