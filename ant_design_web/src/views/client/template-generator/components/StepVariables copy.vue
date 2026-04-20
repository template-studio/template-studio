<template>
  <div class="step-variables">
    <div class="variables-content">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-container">
        <a-spin size="large" tip="正在加载模板变量信息..." />
      </div>

      <!-- 变量配置表单 -->
      <div v-else class="variables-form">
        <div class="form-header">
          <div class="form-title-section">
            <h2 class="form-title">配置项目变量</h2>
            <p class="form-desc">请填写以下信息来配置您的项目</p>
          </div>

          <!-- 模式切换 -->
          <div class="mode-tabs">
            <a-segmented v-model:value="currentMode" :options="[{ label: '普通模式', value: 'normal' }, { label: '高级模式', value: 'advanced' }]" />
          </div>
        </div>

        <!-- 普通模式 -->
        <div v-if="currentMode === 'normal'" class="normal-mode">
          <!-- 自定义变量 -->
          <div class="variable-section" v-if="customVariables.length > 0">
            <h3 class="section-title">
              <CodeSlash style="font-size: 16px" />
              自定义变量
              <span class="section-subtitle"
                >({{ variableStatistics?.totalCustomVariables || 0 }} 个变量)</span
              >
            </h3>
            <div class="variable-grid">
              <div
                v-for="variable in customVariables"
                :key="variable.name"
                class="variable-item"
                :class="{
                  'full-width':
                    variable.variableType === 'object' || variable.variableType === 'object_arr',
                }"
              >
                <label>
                  <span class="variable-name">{{ variable.name }}</span>
                  <span
                    v-if="variable.description && variable.description !== '模板变量'"
                    class="variable-desc-inline"
                    >- {{ variable.description }}</span
                  >
                  <span v-if="variable.isRequired" class="required-mark">*</span>
                  <span class="variable-type-badge">{{
                    getVariableTypeLabel(variable.variableType)
                  }}</span>
                  <span v-if="variable.fileCount > 0" class="usage-indicator">
                    在{{ variable.fileCount }}个文件中使用
                  </span>
                </label>

                <!-- 根据变量类型渲染不同的输入组件 -->
                <!-- 字符串类型 -->
                <a-input
                  v-if="variable.variableType === 'string'"
                  v-model:value="formData[variable.name]"
                  :placeholder="variable.description || '请输入字符串'"
                  :status="getValidationStatus(variable.name)"
                />

                <!-- 数字类型 -->
                <a-input-number
                  v-else-if="variable.variableType === 'number'"
                  v-model:value="formData[variable.name]"
                  :placeholder="variable.description || '请输入数字'"
                  :status="getValidationStatus(variable.name)"
                  style="width: 100%"
                />

                <!-- 布尔类型 -->
                <a-switch
                  v-else-if="variable.variableType === 'boolean'"
                  v-model:checked="formData[variable.name]"
                  checked-children="是"
                  un-checked-children="否"
                />

                <!-- 列表类型 -->
                <div v-else-if="variable.variableType === 'list'" class="list-input">
                  <a-select
                    v-model:value="formData[variable.name]"
                    mode="tags"
                    :placeholder="variable.description || '按回车添加项目'"
                  />
                  <div class="input-hint">按回车添加项目，点击标签删除</div>
                </div>

                <!-- 对象类型 -->
                <div v-else-if="variable.variableType === 'object'" class="object-input">
                  <a-textarea
                    v-model:value="formData[variable.name]"
                    :placeholder="variable.description || '请输入JSON格式的对象'"
                    :status="getValidationStatus(variable.name)"
                    :auto-size="{ minRows: 3, maxRows: 6 }"
                  />
                  <div class="input-hint">请输入有效的JSON格式，例如: {"key": "value"}</div>
                </div>

                <!-- 对象数组类型 -->
                <div v-else-if="variable.variableType === 'object_arr'" class="object-array-input">
                  <a-textarea
                    v-model:value="formData[variable.name]"
                    :placeholder="variable.description || '请输入JSON格式的对象数组'"
                    :status="getValidationStatus(variable.name)"
                    :auto-size="{ minRows: 4, maxRows: 8 }"
                  />
                  <div class="input-hint"
                    >请输入有效的JSON格式，例如: [{"name": "字段1", "type": "string"}, {"name":
                    "字段2", "type": "number"}]</div
                  >
                </div>

                <!-- 其他类型（兼容旧版本） -->
                <a-input
                  v-else
                  v-model:value="formData[variable.name]"
                  :placeholder="variable.description || '请输入值'"
                  :status="getValidationStatus(variable.name)"
                />

                <div class="variable-desc">
                  {{ variable.description }}
                  <div v-if="variable.files && variable.files.length > 0" class="usage-files">
                    使用文件: {{ variable.files.join(', ') }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 高级模式 -->
        <div v-else-if="currentMode === 'advanced'" class="advanced-mode">
          <div class="advanced-editor-container">
            <div class="editor-header">
              <div class="editor-info">
                <span class="editor-title">JSON 变量编辑器</span>
                <span class="editor-desc">直接编辑 JSON 格式的变量数据</span>
              </div>
              <div class="editor-actions">
                <a-button size="small" @click="formatJSON">
                  <template #icon>
                    <CodeSlash />
                  </template>
                  格式化
                </a-button>
                <a-button size="small" @click="generateFromNormal">
                  <template #icon>
                    <RefreshOutline />
                  </template>
                  从普通模式生成
                </a-button>
              </div>
            </div>

            <div class="json-editor-wrapper">
              <div ref="jsonEditorContainer" class="json-editor"></div>
            </div>

            <div class="editor-footer">
              <div class="editor-status">
                <span v-if="jsonValid" class="status-valid">
                  <CheckmarkCircleOutline style="font-size: 14px" />
                  JSON 格式正确
                </span>
                <span v-else class="status-invalid">
                  <AlertCircleOutline style="font-size: 14px" />
                  JSON 格式错误: {{ jsonError }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部操作 -->
    <div class="step-actions">
      <a-button @click="$emit('prev')">
        <template #icon>
          <ArrowBack />
        </template>
        上一步
      </a-button>

      <a-button type="primary" @click="handleNext" :disabled="!isFormValid">
        下一步
        <template #icon>
          <ArrowForward />
        </template>
      </a-button>
    </div>
  </div>
</template>

<script setup>
  import { ref, computed, watch, onMounted, nextTick } from 'vue';
  import {
    Person,
    Settings,
    CodeSlash,
    ArrowBack,
    ArrowForward,
    RefreshOutline,
    CheckmarkCircleOutline,
    AlertCircleOutline,
  } from '@/icons/ionicons5';
  import { getTemplateExpose } from '@/api/templateExpose';

  // CodeMirror 相关导入
  import { EditorView } from '@codemirror/view';
  import { EditorState } from '@codemirror/state';
  import { json } from '@codemirror/lang-json';
  import { dracula } from '@uiw/codemirror-theme-dracula';

  const props = defineProps({
    templateInfo: {
      type: Object,
      default: null,
    },
    variables: {
      type: Object,
      default: () => ({}),
    },
  });

  const emit = defineEmits(['prev', 'next', 'update-variables']);

  // 表单数据
  const formData = ref({});

  // 变量数据
  const customVariables = ref([]);
  const builtinVariables = ref([]);
  const templateFunctions = ref([]);
  const variableStatistics = ref(null);
  const loading = ref(false);

  // 模式切换相关
  const currentMode = ref('normal');

  // 高级模式相关
  const jsonEditorContainer = ref(null);
  let jsonEditor = null;
  const jsonValid = ref(true);
  const jsonError = ref('');

  // 动态验证规则
  const getValidationRules = () => {
    const rules = {};

    // 为必填的自定义变量添加验证规则
    customVariables.value.forEach((variable) => {
      if (variable.isRequired) {
        rules[variable.name] = {
          required: true,
          message: `${variable.description || variable.name}不能为空`,
        };

        // 如果有验证正则表达式，添加模式验证
        if (variable.validationRegex) {
          try {
            rules[variable.name].pattern = new RegExp(variable.validationRegex);
            rules[variable.name].message = `${variable.description || variable.name}格式不正确`;
          } catch (e) {
            console.warn('无效的验证正则表达式:', variable.validationRegex);
          }
        }
      }
    });

    return rules;
  };

  // 验证状态
  const getValidationStatus = (fieldName) => {
    const rules = getValidationRules();
    const rule = rules[fieldName];
    if (!rule) return undefined;

    const value = formData.value[fieldName];

    if (rule.required && !value) {
      return 'error';
    }

    if (rule.pattern && value && !rule.pattern.test(value)) {
      return 'error';
    }

    // 对象和对象数组类型的JSON格式验证
    const variable = customVariables.value.find((v) => v.name === fieldName);
    if ((variable?.variableType === 'object' || variable?.variableType === 'object_arr') && value) {
      try {
        JSON.parse(value);
      } catch {
        return 'error';
      }
    }

    return undefined;
  };

  // 表单是否有效
  const isFormValid = computed(() => {
    // 高级模式下，只需要验证 JSON 格式是否正确
    if (currentMode.value === 'advanced') {
      return jsonValid.value;
    }

    // 普通模式下的验证逻辑
    const rules = getValidationRules();

    for (const [fieldName, rule] of Object.entries(rules)) {
      const value = formData.value[fieldName];
      const variable = customVariables.value.find((v) => v.name === fieldName);

      // 必填验证
      if (rule.required) {
        if (variable?.variableType === 'list') {
          if (!Array.isArray(value) || value.length === 0) {
            return false;
          }
        } else if (variable?.variableType === 'boolean') {
          // 布尔类型不需要必填验证，因为它总是有值（true/false）
        } else if (!value) {
          return false;
        }
      }

      // 正则验证
      if (rule.pattern && value && !rule.pattern.test(value)) {
        return false;
      }

      // 对象和对象数组类型的JSON格式验证
      if (
        (variable?.variableType === 'object' || variable?.variableType === 'object_arr') &&
        value
      ) {
        try {
          JSON.parse(value);
        } catch {
          return false;
        }
      }
    }

    return true;
  });

  // 获取输入类型（兼容旧版本）
  const getInputType = (variableType) => {
    switch (variableType) {
      case 'email':
        return 'email';
      case 'number':
        return 'number';
      case 'textarea':
        return 'textarea';
      case 'password':
        return 'password';
      default:
        return 'text';
    }
  };

  // 获取变量类型标签
  const getVariableTypeLabel = (variableType) => {
    switch (variableType) {
      case 'string':
        return '字符串';
      case 'number':
        return '数字';
      case 'boolean':
        return '布尔值';
      case 'list':
        return '列表';
      case 'object':
        return '对象';
      case 'object_arr':
        return '对象数组';
      default:
        return '文本';
    }
  };

  // 根据变量类型获取默认值
  const getDefaultValue = (variable) => {
    if (variable.defaultValue !== undefined && variable.defaultValue !== null) {
      // 如果有设置默认值，根据类型转换
      switch (variable.variableType) {
        case 'boolean':
          return variable.defaultValue === 'true' || variable.defaultValue === true;
        case 'number':
          return Number(variable.defaultValue) || 0;
        case 'list':
          if (typeof variable.defaultValue === 'string') {
            try {
              const parsed = JSON.parse(variable.defaultValue);
              return Array.isArray(parsed) ? parsed : [variable.defaultValue];
            } catch {
              return variable.defaultValue.split(',').map((s) => s.trim());
            }
          }
          return Array.isArray(variable.defaultValue) ? variable.defaultValue : [];
        case 'object':
          if (typeof variable.defaultValue === 'string') {
            try {
              return JSON.parse(variable.defaultValue);
            } catch {
              return variable.defaultValue;
            }
          }
          return variable.defaultValue;
        default:
          return String(variable.defaultValue);
      }
    }

    // 没有默认值时，根据类型返回默认值
    switch (variable.variableType) {
      case 'boolean':
        return false;
      case 'number':
        return 0;
      case 'list':
        return [];
      case 'object':
        return '{}';
      default:
        return '';
    }
  };

  // 加载模板变量数据
  const loadTemplateVariables = async () => {
    if (!props.templateInfo?.id) {
      return;
    }
    loading.value = true;
    try {
      // 获取模板变量定义
      let userDefinedVariables = [];
      try {
        const exposeRes = await getTemplateExpose({ templateId: props.templateInfo.id });

        if (exposeRes.data?.code === 0 && exposeRes.data?.data?.templateExpose) {
          const fieldSchemaJson = exposeRes.data.data.templateExpose.fieldSchemaJson;
          if (fieldSchemaJson) {
            try {
              const parsedSchema = JSON.parse(fieldSchemaJson);
              userDefinedVariables = convertSchemaToCustomVariables(parsedSchema);
            } catch (parseError) {
              console.error('解析变量定义失败:', parseError);
            }
          }
        }
      } catch (error) {
        console.error('获取模板变量定义失败:', error);
      }

      // 使用用户定义的变量
      customVariables.value = userDefinedVariables;
      builtinVariables.value = [];
      templateFunctions.value = [];
      variableStatistics.value = null;

      // 初始化表单数据
      const newFormData = { ...formData.value };

      // 初始化自定义变量
      customVariables.value.forEach((variable) => {
        if (!(variable.name in newFormData)) {
          newFormData[variable.name] = getDefaultValue(variable);
        }
      });

      // 初始化内置变量
      builtinVariables.value.forEach((variable) => {
        const fieldName = variable.name.toLowerCase();
        if (!newFormData[fieldName]) {
          newFormData[fieldName] = '';
        }
      });

      // 一次性更新表单数据
      formData.value = newFormData;
    } catch (error) {
      console.error('加载模板变量失败:', error);
      customVariables.value = [];
      builtinVariables.value = [];
      templateFunctions.value = [];
      variableStatistics.value = null;
    } finally {
      loading.value = false;
    }
  };

  // 合并用户定义的变量和检测到的变量
  const mergeVariables = (userDefined, detected) => {
    const mergedMap = new Map();

    // 首先添加用户定义的变量（有完整的类型和验证信息）
    userDefined.forEach((variable) => {
      mergedMap.set(variable.name, {
        ...variable,
        isUserDefined: true,
        files: [],
        contexts: [],
        fileCount: 0,
      });
    });

    // 然后添加或更新检测到的变量信息
    detected.forEach((detectedVar) => {
      if (mergedMap.has(detectedVar.name)) {
        // 如果用户已定义该变量，更新使用信息
        const existing = mergedMap.get(detectedVar.name);
        existing.files = detectedVar.files || [];
        existing.contexts = detectedVar.contexts || [];
        existing.fileCount = (detectedVar.files || []).length;
        existing.isDetected = true;
      } else {
        // 如果用户未定义该变量，添加为新变量
        mergedMap.set(detectedVar.name, {
          name: detectedVar.name,
          variableType: convertTypeToOldFormat(detectedVar.type),
          description: detectedVar.suggestions || `模板中使用的 ${detectedVar.name} 变量`,
          defaultValue: getDefaultValueByType(detectedVar.type),
          isRequired: false,
          isUserDefined: false,
          isDetected: true,
          files: detectedVar.files || [],
          contexts: detectedVar.contexts || [],
          fileCount: (detectedVar.files || []).length,
        });
      }
    });

    return Array.from(mergedMap.values());
  };

  // 根据类型获取默认值
  const getDefaultValueByType = (type) => {
    switch (type) {
      case 'boolean':
        return false;
      case 'number':
      case 'integer':
        return 0;
      case 'array':
        return [];
      case 'object':
        return '{}';
      default:
        return '';
    }
  };

  // 将schema格式转换为自定义变量列表格式
  const convertSchemaToCustomVariables = (schema) => {
    const variablesList = [];

    if (!schema || typeof schema !== 'object') {
      return variablesList;
    }

    Object.entries(schema).forEach(([key, variable]) => {
      if (variable && typeof variable === 'object') {
        variablesList.push({
          name: key,
          variableType: convertTypeToOldFormat(variable.type),
          description: variable.description || variable.title || key,
          defaultValue: variable.default,
          isRequired: variable.required || false,
          validationRegex: variable.pattern,
        });
      }
    });

    return variablesList;
  };

  // 将新的类型格式转换为旧的格式
  const convertTypeToOldFormat = (newType) => {
    switch (newType) {
      case 'string':
        return 'string';
      case 'integer':
      case 'number':
        return 'number';
      case 'boolean':
        return 'boolean';
      case 'array':
        return 'list';
      case 'object':
        return 'object';
      case 'object_arr':
        return 'object_arr'; // 保持 object_arr 类型，不转换为 object
      default:
        return 'string';
    }
  };

  // 将对象/数组类型的数据转换为普通模式下的字符串显示
  const convertForNormalMode = (data) => {
    const converted = { ...data };

    customVariables.value.forEach((variable) => {
      const value = converted[variable.name];

      if (value !== undefined && value !== null) {
        switch (variable.variableType) {
          case 'object':
            // 对象类型转换为JSON字符串
            if (typeof value === 'object' && !Array.isArray(value)) {
              converted[variable.name] = JSON.stringify(value, null, 2);
            }
            break;
          case 'object_arr':
            // 对象数组类型转换为JSON字符串
            if (typeof value === 'object') {
              converted[variable.name] = JSON.stringify(value, null, 2);
            }
            break;
          case 'list':
            // 列表类型保持为数组（a-select mode="tags"组件需要数组）
            if (Array.isArray(value)) {
              converted[variable.name] = value;
            } else if (typeof value === 'string') {
              try {
                const parsed = JSON.parse(value);
                converted[variable.name] = Array.isArray(parsed) ? parsed : [value];
              } catch {
                converted[variable.name] = [value];
              }
            }
            break;
        }
      }
    });

    return converted;
  };

  // 将普通模式的数据转换为适合高级模式的格式
  const convertForAdvancedMode = (data) => {
    const converted = { ...data };

    customVariables.value.forEach((variable) => {
      const value = converted[variable.name];
      if (value !== undefined && value !== null) {
        switch (variable.variableType) {
          case 'object':
            // 字符串转换为对象
            if (typeof value === 'string') {
              try {
                converted[variable.name] = JSON.parse(value);
              } catch {
                converted[variable.name] = {};
              }
            }
            break;
          case 'object_arr':
            // 字符串转换为对象数组
            if (typeof value === 'string') {
              try {
                converted[variable.name] = JSON.parse(value);
              } catch {
                converted[variable.name] = [];
              }
            }
            break;
        }
      }
    });

    return converted;
  };

  // 监听模式切换，转换数据格式
  watch(currentMode, (newMode, oldMode) => {
    if (oldMode && newMode !== oldMode) {
      if (newMode === 'normal') {
        // 切换到普通模式时，需要从JSON编辑器获取最新数据
        let dataToConvert = formData.value;

        // 如果从高级模式切换过来，优先使用编辑器中的数据
        if (oldMode === 'advanced' && jsonEditor) {
          try {
            const editorContent = jsonEditor.state.doc.toString();
            const jsonData = JSON.parse(editorContent);
            dataToConvert = jsonData;
          } catch (error) {
            console.warn('JSON编辑器内容解析失败，使用formData:', error);
          }
        }

        const converted = convertForNormalMode(dataToConvert);

        // 使用 nextTick 确保在下一个事件循环中更新，避免响应式问题
        nextTick(() => {
          // 逐个更新字段，确保响应式更新
          Object.keys(converted).forEach((key) => {
            formData.value[key] = converted[key];
          });
        });
      } else if (newMode === 'advanced') {
        // 切换到高级模式，将字符串转换为对象
        const converted = convertForAdvancedMode(formData.value);

        nextTick(() => {
          Object.keys(converted).forEach((key) => {
            formData.value[key] = converted[key];
          });
        });
      }
    }
  });

  // 监听表单变化，使用防抖避免频繁触发
  let updateTimeout = null;
  watch(
    formData,
    (newData) => {
      if (updateTimeout) {
        clearTimeout(updateTimeout);
      }
      updateTimeout = setTimeout(() => {
        emit('update-variables', newData);
      }, 100);
    },
    { deep: true }
  );

  // 初始化表单数据，只在组件初始化时执行一次
  let isInitialized = false;
  watch(
    () => props.variables,
    (newVariables) => {
      if (!isInitialized && newVariables && Object.keys(newVariables).length > 0) {
        formData.value = { ...formData.value, ...newVariables };
        isInitialized = true;
      }
    },
    { immediate: true }
  );

  // 监听templateInfo变化，当有数据时加载变量
  let lastTemplateId = null;
  watch(
    () => props.templateInfo,
    (newTemplateInfo) => {
      if (newTemplateInfo?.id && newTemplateInfo.id !== lastTemplateId) {
        lastTemplateId = newTemplateInfo.id;
        // 延迟一点加载，确保组件完全渲染
        setTimeout(() => {
          loadTemplateVariables();
        }, 100);
      }
    },
    { immediate: false }
  );

  // 组件挂载时不立即加载，等待 templateInfo 传入
  onMounted(() => {
    // 如果已经有 templateInfo，则加载数据
    if (props.templateInfo?.id) {
      lastTemplateId = props.templateInfo.id;
      setTimeout(() => {
        loadTemplateVariables();
      }, 100);
    }
  });

  // 下一步
  const handleNext = () => {
    if (currentMode.value === 'normal') {
      if (isFormValid.value) {
        // 普通模式下，将字符串类型的对象转换回对象格式
        const convertedData = convertForAdvancedMode(formData.value);
        emit('update-variables', convertedData);
        emit('next');
      }
    } else {
      // 高级模式下，从编辑器获取数据并验证
      if (jsonValid.value && jsonEditor) {
        try {
          const editorContent = jsonEditor.state.doc.toString();
          const jsonData = JSON.parse(editorContent);
          emit('update-variables', jsonData);
          emit('next');
        } catch (error) {
          // JSON 解析错误已经在 validateJSON 中处理
        }
      }
    }
  };

  // 初始化 JSON 编辑器
  const initJsonEditor = async () => {
    if (!jsonEditorContainer.value) return;

    // 尝试生成完整的测试数据作为初始内容
    let initialData = formData.value;
    try {
      const exposeRes = await getTemplateExpose({ templateId: props.templateInfo.id });
      if (exposeRes.data?.code === 0 && exposeRes.data?.data?.templateExpose) {
        const fieldSchemaJson = exposeRes.data.data.templateExpose.fieldSchemaJson;
        if (fieldSchemaJson) {
          const originalSchema = JSON.parse(fieldSchemaJson);
          // 使用增强的生成函数，传入现有表单数据以保留用户输入
          initialData = generateTestDataFromSchema(originalSchema, formData.value);
        }
      }
    } catch (error) {
      console.error('获取 schema 失败，使用表单数据:', error);
    }

    const initialContent = JSON.stringify(initialData, null, 2);

    const state = EditorState.create({
      doc: initialContent,
      extensions: [
        dracula,
        json(),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            validateJSON();
          }
        }),
        EditorView.theme({
          '&': { height: '400px' },
          '.cm-scroller': {
            overflow: 'auto !important',
            height: '100% !important',
          },
        }),
      ],
    });

    jsonEditor = new EditorView({
      state,
      parent: jsonEditorContainer.value,
    });

    validateJSON();
  };

  // 验证 JSON 格式
  const validateJSON = () => {
    if (!jsonEditor) return;

    try {
      const content = jsonEditor.state.doc.toString();
      JSON.parse(content);
      jsonValid.value = true;
      jsonError.value = '';
    } catch (error) {
      jsonValid.value = false;
      jsonError.value = error.message;
    }
  };

  // 格式化 JSON
  const formatJSON = () => {
    if (!jsonEditor) return;

    try {
      const content = jsonEditor.state.doc.toString();
      const parsed = JSON.parse(content);
      const formatted = JSON.stringify(parsed, null, 2);

      jsonEditor.dispatch({
        changes: {
          from: 0,
          to: jsonEditor.state.doc.length,
          insert: formatted,
        },
      });
    } catch (error) {
      // JSON 格式错误，不执行格式化
    }
  };

  // 从普通模式生成 JSON（支持嵌套结构）
  const generateFromNormal = async () => {
    if (!jsonEditor) return;

    try {
      // 获取当前编辑器中的数据作为现有数据
      let existingData = {};
      try {
        const editorContent = jsonEditor.state.doc.toString();
        existingData = JSON.parse(editorContent);
      } catch (e) {
        existingData = formData.value; // 如果编辑器内容无效，使用表单数据
      }

      // 重新获取原始 schema 定义
      const exposeRes = await getTemplateExpose({ templateId: props.templateInfo.id });
      let originalSchema = null;

      if (exposeRes.data?.code === 0 && exposeRes.data?.data?.templateExpose) {
        const fieldSchemaJson = exposeRes.data.data.templateExpose.fieldSchemaJson;
        if (fieldSchemaJson) {
          originalSchema = JSON.parse(fieldSchemaJson);
        }
      }

      // 生成包含嵌套结构的完整测试数据，保留现有数据
      const testData = originalSchema
        ? generateTestDataFromSchema(originalSchema, existingData)
        : generateTestDataFromVariables();

      const jsonContent = JSON.stringify(testData, null, 2);

      jsonEditor.dispatch({
        changes: {
          from: 0,
          to: jsonEditor.state.doc.length,
          insert: jsonContent,
        },
      });
    } catch (error) {
      console.error('生成测试数据失败:', error);
      // 回退到简单生成
      const testData = generateTestDataFromVariables();
      const jsonContent = JSON.stringify(testData, null, 2);

      jsonEditor.dispatch({
        changes: {
          from: 0,
          to: jsonEditor.state.doc.length,
          insert: jsonContent,
        },
      });
    }
  };

  // 从原始 schema 生成测试数据（参考模板编辑页面）
  const generateTestDataFromSchema = (schema, existingData = {}) => {
    const data = {};

    if (!schema || typeof schema !== 'object') {
      return data;
    }

    Object.entries(schema).forEach(([key, variable]) => {
      if (!variable || typeof variable !== 'object') {
        return;
      }

      // 如果现有数据中有该字段，根据类型进行合并
      if (existingData && existingData.hasOwnProperty(key)) {
        switch (variable.type) {
          case 'object':
            if (
              variable.properties &&
              typeof existingData[key] === 'object' &&
              existingData[key] !== null
            ) {
              // 对象类型递归合并
              data[key] = generateTestDataFromSchema(variable.properties, existingData[key]);
            } else {
              data[key] = existingData[key];
            }
            break;
          case 'object_arr':
            if (Array.isArray(existingData[key]) && existingData[key].length > 0) {
              // 对象数组有内容时保留现有数据
              data[key] = existingData[key];
            } else {
              // 如果现有数据类型不匹配或为空数组，重新生成
              if (variable.items && variable.items.properties) {
                const itemData = generateTestDataFromSchema(variable.items.properties);
                data[key] = [itemData, { ...itemData }];
              } else {
                data[key] = [];
              }
            }
            break;
          default:
            // 其他类型直接使用现有数据
            data[key] = existingData[key];
        }
      } else {
        // 没有现有数据，生成新的默认数据
        switch (variable.type) {
          case 'string':
            // 使用更有意义的默认值
            if (variable.default !== undefined && variable.default !== '') {
              data[key] = variable.default;
            } else {
              // 根据字段名生成更合理的默认值
              if (key === 'name') {
                data[key] = '示例name';
              } else if (key === 'type') {
                data[key] = '示例type';
              } else if (key === 'dbName') {
                data[key] = 'test';
              } else if (key === 'Author') {
                data[key] = 'admin';
              } else if (key === 'module') {
                data[key] = 'github.com/github/test';
              } else if (key === 'ip') {
                data[key] = '127.0.0.1';
              } else {
                data[key] = `示例${key}`;
              }
            }
            break;
          case 'integer':
            data[key] = variable.default !== undefined ? variable.default : 42;
            break;
          case 'number':
            data[key] = variable.default !== undefined ? variable.default : 3.14;
            break;
          case 'boolean':
            data[key] = variable.default !== undefined ? variable.default : true;
            break;
          case 'array':
            data[key] = variable.default || ['item1', 'item2'];
            break;
          case 'object':
            if (variable.properties) {
              data[key] = generateTestDataFromSchema(variable.properties);
            } else {
              data[key] = {};
            }
            break;
          case 'object_arr':
            if (variable.items && variable.items.properties) {
              const itemData = generateTestDataFromSchema(variable.items.properties);
              data[key] = [itemData, { ...itemData }]; // 生成两个示例项
            } else {
              data[key] = [];
            }
            break;
          case 'enum':
            data[key] =
              variable.enum && variable.enum.length > 0 ? variable.enum[0] : variable.default || '';
            break;
          case 'secret':
            data[key] = variable.default !== undefined ? variable.default : '***保密信息***';
            break;
          default:
            data[key] = variable.default !== undefined ? variable.default : '';
        }
      }
    });

    return data;
  };

  // 基于变量定义生成测试数据
  const generateTestDataFromVariables = () => {
    const testData = {};

    // 首先从表单数据开始
    Object.assign(testData, formData.value);

    // 然后为每个定义的变量生成适当的测试数据
    customVariables.value.forEach((variable) => {
      if (!testData.hasOwnProperty(variable.name)) {
        testData[variable.name] = generateValueByType(variable);
      }
    });

    return testData;
  };

  // 根据变量类型生成对应的测试值
  const generateValueByType = (variable) => {
    // 如果有默认值，优先使用
    if (variable.defaultValue !== undefined && variable.defaultValue !== null) {
      return variable.defaultValue;
    }

    switch (variable.variableType) {
      case 'string':
        return `示例${variable.name}`;
      case 'integer':
      case 'number':
        return 42;
      case 'boolean':
        return true;
      case 'list':
      case 'array':
        return ['item1', 'item2'];
      case 'object':
        return generateNestedObject(variable);
      case 'object_arr':
        return generateObjectArray(variable);
      default:
        return '';
    }
  };

  // 生成嵌套对象结构（参考模板编辑页面的逻辑）
  const generateNestedObject = (variable) => {
    // 检查是否有嵌套属性定义
    if (variable.properties) {
      const obj = {};
      Object.entries(variable.properties).forEach(([key, prop]) => {
        obj[key] = generateValueByType(prop);
      });
      return obj;
    }

    // 如果没有具体的属性定义，返回一个示例对象
    return {
      示例属性: '示例值',
      数字属性: 123,
      布尔属性: true,
    };
  };

  // 处理对象数组类型的生成
  const generateObjectArray = (variable) => {
    if (variable.items && variable.items.properties) {
      const itemData = generateNestedObject(variable.items);
      return [itemData, { ...itemData }]; // 生成两个示例项
    }

    return [
      {
        id: 1,
        name: '示例项目1',
        value: '值1',
      },
      {
        id: 2,
        name: '示例项目2',
        value: '值2',
      },
    ];
  };

  // 监听模式切换
  watch(currentMode, (newMode) => {
    if (newMode === 'advanced') {
      // 切换到高级模式时，初始化编辑器
      nextTick(() => {
        initJsonEditor();
      });
    } else if (newMode === 'normal' && jsonEditor) {
      // 切换到普通模式时，同步数据
      try {
        const content = jsonEditor.state.doc.toString();
        const jsonData = JSON.parse(content);
        formData.value = { ...formData.value, ...jsonData };
      } catch (error) {
        // JSON 格式错误，不同步数据
      }
    }
  });
</script>

<style scoped>
  .step-variables {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .variables-content {
    flex: 1;
    padding: 40px;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .loading-container {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
  }

  .variables-form {
    width: 100%;
    max-width: 800px;
    margin: 0 auto;
  }

  .form-title {
    font-size: 24px;
    font-weight: bold;
    color: #333;
    margin-bottom: 8px;
  }

  .form-desc {
    color: #666;
    margin-bottom: 32px;
  }

  .variable-section {
    margin-bottom: 40px;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 18px;
    font-weight: bold;
    color: #333;
    margin-bottom: 20px;
    padding-bottom: 12px;
    border-bottom: 1px solid #f0f0f0;
  }

  .section-subtitle {
    font-size: 14px;
    font-weight: normal;
    color: #666;
    margin-left: 8px;
  }

  .variable-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
  }

  .variable-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* 对象和对象数组类型占据完整一行 */
  .variable-item:has(.object-input),
  .variable-item:has(.object-array-input) {
    grid-column: 1 / -1;
  }

  /* 如果浏览器不支持 :has 选择器，使用类名方式 */
  .variable-item.full-width {
    grid-column: 1 / -1;
  }

  /* 对象输入框样式 */
  .object-input,
  .object-array-input {
    width: 100%;
  }

  .object-input .ant-input,
  .object-array-input .ant-input {
    width: 100%;
  }

  .input-hint {
    font-size: 12px;
    color: #999;
    margin-top: 4px;
    font-style: italic;
  }

  .builtin-variable {
    background: #f8f9fa;
    padding: 16px;
    border-radius: 8px;
    border: 1px solid #e9ecef;
  }

  .variable-item label {
    font-weight: bold;
    color: #333;
    font-size: 14px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 4px;
  }

  .variable-name {
    font-weight: bold;
    color: #333;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  }

  .variable-desc-inline {
    font-weight: normal;
    color: #666;
    font-size: 13px;
  }

  .variable-desc {
    font-size: 12px;
    color: #999;
    line-height: 1.4;
  }

  .usage-count {
    font-size: 12px;
    color: #18a058;
    font-weight: normal;
  }

  .usage-files {
    font-size: 11px;
    color: #666;
    margin-top: 4px;
    font-style: italic;
  }

  .usage-info {
    font-size: 11px;
    color: #18a058;
    margin-top: 4px;
    font-style: italic;
  }

  .required-mark {
    color: #d03050;
    margin-left: 4px;
  }

  .collapse-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: bold;
    color: #333;
  }

  .function-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .function-item {
    padding: 16px;
    background: #f8f9fa;
    border-radius: 8px;
    border: 1px solid #e9ecef;
  }

  .function-info {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .function-name {
    font-weight: bold;
    color: #333;
    font-size: 14px;
  }

  .function-desc {
    font-size: 12px;
    color: #666;
    line-height: 1.4;
    margin-bottom: 8px;
  }

  .variable-type-badge {
    display: inline-block;
    background: #f0f0f0;
    color: #666;
    font-size: 11px;
    font-weight: normal;
    padding: 2px 6px;
    border-radius: 3px;
    margin-left: 8px;
  }

  .list-input {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .object-input {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .input-hint {
    font-size: 11px;
    color: #999;
    font-style: italic;
  }

  /* 为不同变量类型的徽章添加颜色 */
  .variable-type-badge {
    background: #e6f7ff;
    color: #1890ff;
  }

  .variable-item:has([data-variable-type='boolean']) .variable-type-badge {
    background: #f6ffed;
    color: #52c41a;
  }

  .variable-item:has([data-variable-type='number']) .variable-type-badge {
    background: #fff7e6;
    color: #fa8c16;
  }

  .variable-item:has([data-variable-type='list']) .variable-type-badge {
    background: #f9f0ff;
    color: #722ed1;
  }

  .variable-item:has([data-variable-type='object']) .variable-type-badge {
    background: #e6fffb;
    color: #13c2c2;
  }

  .usage-indicator {
    font-size: 11px;
    color: #18a058;
    font-weight: normal;
    margin-left: 8px;
    background: #f0f9f0;
    padding: 2px 6px;
    border-radius: 3px;
  }

  /* 表单头部样式 */
  .form-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 32px;
    gap: 20px;
  }

  .form-title-section {
    flex: 1;
  }

  .mode-tabs {
    flex-shrink: 0;
  }

  /* 高级模式样式 */
  .advanced-mode {
    width: 100%;
  }

  .advanced-editor-container {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    overflow: hidden;
    background: #fff;
  }

  .editor-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    background: #f8f9fa;
    border-bottom: 1px solid #e0e0e0;
  }

  .editor-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .editor-title {
    font-weight: 600;
    font-size: 16px;
    color: #333;
  }

  .editor-desc {
    font-size: 13px;
    color: #666;
  }

  .editor-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .json-editor-wrapper {
    height: 400px;
    overflow: hidden;
  }

  .json-editor {
    height: 100%;
    border: none;
  }

  .editor-footer {
    padding: 12px 20px;
    background: #f8f9fa;
    border-top: 1px solid #e0e0e0;
  }

  .editor-status {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-valid {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #18a058;
    font-size: 13px;
  }

  .status-invalid {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #d03050;
    font-size: 13px;
  }

  .step-actions {
    padding: 16px 30px;
    border-top: 1px solid #f0f0f0;
    background: #fafafa;
    display: flex;
    justify-content: space-between;
  }
</style>
