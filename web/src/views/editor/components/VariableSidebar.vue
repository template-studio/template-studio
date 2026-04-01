<template>
  <div class="variable-sidebar" :style="{ width: sidebarWidth + 'px' }">
    <!-- 头部：标题 + 操作按钮 -->
    <div class="sidebar-header">
      <div class="header-left">
        <n-icon size="18" color="#722ed1">
          <AppsOutline />
        </n-icon>
        <span class="header-title">变量</span>
      </div>
      <div class="header-actions">
        <!-- Variable Studio 按钮 -->
        <n-button
          size="small"
          @click="emit('show-quick-design')"
          quaternary
          class="action-btn"
          title="Variable Studio（可视化设计，所见即所得）"
        >
          <template #icon>
            <n-icon size="14">
              <ConstructOutline />
            </n-icon>
          </template>
        </n-button>

        <!-- 测试数据按钮 -->
        <n-button
          size="small"
          @click="emit('show-test-data')"
          quaternary
          class="action-btn"
          title="测试数据"
        >
          <template #icon>
            <n-icon size="14">
              <svg viewBox="0 0 24 24" width="14" height="14">
                <path
                  fill="currentColor"
                  d="M9 3L7 17c0 2.76 2.24 5 5 5s5-2.24 5-5L15 3H9zm2 2h2v2h-2V5zm0 4h2v2h-2V9zm0 4h2v2h-2v-2z"
                />
              </svg>
            </n-icon>
          </template>
        </n-button>
      </div>
    </div>

    <!-- 内部垂直 Tabs 和内容 -->
    <div class="sidebar-content">
      <!-- 垂直 Tabs -->
      <div class="vertical-tabs">
        <div
          v-for="tab in variableTabs"
          :key="tab.key"
          :class="['tab-item', { active: activeTab === tab.key, 'preset-tab': tab.isPreset }]"
          @click="activeTab = tab.key"
          :title="tab.label"
        >
          <n-icon size="16">
            <component :is="tab.icon" />
          </n-icon>
          <span class="tab-label">{{ tab.label }}</span>
          <span v-if="tab.isPreset" class="tab-badge">订阅</span>
        </div>
      </div>

      <!-- 变量列表内容区域 -->
      <!-- 模板语法 Tab -->
      <div v-show="activeTab === 'syntax'" class="tab-pane">
        <div
          v-for="category in templateSyntaxCategories"
          :key="category.name"
          class="variable-category"
        >
          <div class="category-divider">
            <span class="category-name">{{ category.name }}</span>
            <span class="category-count">({{ category.syntaxes.length }})</span>
          </div>
          <div class="variable-items">
            <div
              v-for="syntax in category.syntaxes"
              :key="syntax.name"
              class="variable-item"
              @click="handleInsertSyntax(syntax)"
              @mouseenter="handleShowSyntaxDetail(syntax, $event)"
              @mouseleave="handleHideFunctionDetail"
            >
              <div class="variable-info">
                <n-icon size="16" color="#1890ff">
                  <CodeSlash />
                </n-icon>
                <span class="variable-name">{{ syntax.display_name || syntax.name }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 内置函数 Tab -->
      <div v-show="activeTab === 'functions'" class="tab-pane">
        <div v-if="loadingFunctions" class="loading-state">
          <n-spin size="small" />
          <span style="margin-left: 8px">加载函数中...</span>
        </div>
        <div v-else>
          <div
            v-for="category in builtinFunctionCategories"
            :key="category.name"
            class="variable-category"
          >
            <div class="category-divider">
              <span class="category-name">{{ category.name }}</span>
              <span class="category-count">({{ category.functions.length }})</span>
            </div>
            <div class="variable-items">
              <div
                v-for="func in category.functions"
                :key="func.name"
                class="variable-item"
                @click="handleInsertFunction(func)"
                @mouseenter="handleShowFunctionDetail(func, $event)"
                @mouseleave="handleHideFunctionDetail"
              >
                <div class="variable-info">
                  <n-icon size="16" color="#52c41a">
                    <ConstructOutline />
                  </n-icon>
                  <span class="variable-name">{{ func.display_name || func.name }}</span>
                </div>
              </div>
            </div>
          </div>

          <div v-if="builtinFunctionCategories.length === 0" class="empty-state">
            <span>暂无可用的内置函数</span>
          </div>
        </div>
      </div>

      <!-- 内置变量 Tab -->
      <div v-show="activeTab === 'builtin'" class="tab-pane">
        <div class="variable-category">
          <div class="category-divider">
            <span class="category-name">快速变量</span>
            <span class="category-count">({{ quickVariables.length }})</span>
          </div>
          <div class="variable-items">
            <div
              v-for="variable in quickVariables"
              :key="variable.name"
              class="variable-item"
              @click="handleInsertVariable(variable.name)"
              :title="`${variable.name} - ${variable.label}`"
            >
              <div class="variable-info">
                <n-icon size="16" color="#2f54eb">
                  <BuildOutline />
                </n-icon>
                <span class="variable-name">{{ variable.label }}</span>
              </div>
              <n-tag size="small" type="info">快速</n-tag>
            </div>
          </div>
        </div>
      </div>

      <!-- 用户变量 Tab -->
      <div v-show="activeTab === 'custom'" class="tab-pane">
        <div v-if="loadingVariableDefinitions" class="loading-state">
          <n-spin size="small" />
          <span style="margin-left: 8px">加载变量定义中...</span>
        </div>

        <div v-else>
          <div v-if="textVariables.length > 0" class="variable-category">
            <div class="category-divider">
              <span class="category-name">变量定义</span>
              <span class="category-count">({{ textVariables.length }})</span>
            </div>
            <div class="variable-items">
              <div
                v-for="variable in textVariables"
                :key="variable.id"
                class="variable-item"
                @click="handleInsertVariableDefinition(variable)"
                :title="`${variable.displayName || variable.name} (${getVariableTypeLabel(
                  variable.variableType
                )})${variable.description ? ' - ' + variable.description : ''}`"
              >
                <div class="variable-info">
                  <n-icon size="16" :color="getVariableTypeColor(variable.variableType)">
                    <component :is="getVariableTypeIcon(variable.variableType)" />
                  </n-icon>
                  <span class="variable-name">{{ variable.displayName || variable.name }}</span>
                </div>
                <n-tag size="small" :type="getVariableTypeTagType(variable.variableType)">
                  {{ getVariableTypeLabel(variable.variableType) }}
                </n-tag>
              </div>
            </div>
          </div>

          <div
            v-if="variableDefinitions.length === 0 && !loadingVariableDefinitions"
            class="empty-state"
          >
            <span>暂无变量定义</span>
          </div>
        </div>
      </div>

      <!-- 订阅管理 Tab -->
      <div v-show="activeTab === 'subscription_manager'" class="tab-pane">
        <div v-if="loadingPresets" class="loading-state">
          <n-spin size="small" />
          <span style="margin-left: 8px">加载中...</span>
        </div>

        <div v-else>
          <div v-if="subscribedPresets.length === 0" class="empty-state">
            <n-empty description="暂无订阅的预设变量" size="small">
              <template #extra>
                <n-button size="small" @click="showSubscribeModal = true">立即订阅</n-button>
              </template>
            </n-empty>
          </div>

          <div v-else>
            <div class="variable-category">
              <div class="category-divider">
                <span class="category-name">已订阅的预设</span>
                <span class="category-count">({{ subscribedPresets.length }})</span>
              </div>
              <div class="variable-items">
                <div
                  v-for="preset in subscribedPresets"
                  :key="preset.presetId"
                  class="variable-item preset-subscription-item"
                >
                  <div class="variable-info" @click="activeTab = `preset_${preset.presetId}`">
                    <n-icon size="16" color="#722ed1">
                      <AppsOutline />
                    </n-icon>
                    <span class="variable-name">{{ preset.presetName }}</span>
                  </div>
                  <n-button
                    size="tiny"
                    quaternary
                    type="error"
                    @click="handleUnsubscribePreset(preset)"
                    :loading="preset.unsubscribing"
                  >
                    取消订阅
                  </n-button>
                </div>
              </div>
              <div class="subscribe-tip">
                <n-button size="small" type="primary" @click="showSubscribeModal = true" block>
                  <template #icon>
                    <n-icon><AddOutline /></n-icon>
                  </template>
                  订阅
                </n-button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 动态订阅预设 Tab -->
      <template v-for="preset in subscribedPresets" :key="`pane_${preset.presetId}`">
        <div v-show="activeTab === `preset_${preset.presetId}`" class="tab-pane">
          <div v-if="loadingPresets" class="loading-state">
            <n-spin size="small" />
            <span style="margin-left: 8px">加载预设变量中...</span>
          </div>

          <div v-else class="variable-category">
            <div class="category-divider">
              <span class="category-name">{{ preset.presetName }}</span>
              <span class="category-count"
                >({{ (presetVariables[`preset_${preset.presetId}`] || []).length }})</span
              >
            </div>
            <div class="variable-items">
              <div
                v-for="variable in presetVariables[`preset_${preset.presetId}`] || []"
                :key="variable.name"
                class="variable-item"
                @click="handleInsertPresetVariable(variable)"
                :title="`${variable.displayName} (${variable.type})${
                  variable.description ? ' - ' + variable.description : ''
                }`"
              >
                <div class="variable-info">
                  <n-icon size="16" color="#9254de">
                    <ExtensionPuzzleOutline />
                  </n-icon>
                  <span class="variable-name">{{ variable.displayName }}</span>
                </div>
                <n-tag size="small" type="info">{{ variable.type }}</n-tag>
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- 函数详情悬浮卡片 -->
    <div
      v-if="functionDetailVisible && selectedFunction"
      class="function-detail-card"
      :style="functionDetailStyle"
      @mouseenter="onDetailPanelEnter"
      @mouseleave="handleHideFunctionDetail"
    >
      <div class="detail-header">
        <div class="detail-title">
          <span class="function-icon">{{ selectedFunction.isParent ? '📁' : '🏷️' }}</span>
          {{
            selectedFunction.displayName || selectedFunction.display_name || selectedFunction.name
          }}
        </div>
        <div class="detail-type">{{
          selectedFunction.type || selectedFunction.return_type || 'field'
        }}</div>
      </div>

      <div class="detail-body">
        <!-- 描述信息 -->
        <div v-if="selectedFunction.description" class="detail-description">
          {{ selectedFunction.description }}
        </div>

        <!-- 变量路径 -->
        <div v-if="selectedFunction.name" class="detail-section">
          <div class="section-label">
            <span class="section-icon">📍</span>
            变量路径
          </div>
          <div class="section-content code-content">
            {{ selectedFunction.name }}
          </div>
        </div>

        <!-- 如果有usage字段，显示使用说明 -->
        <div v-if="selectedFunction.usage" class="detail-section">
          <div class="section-label">
            <span class="section-icon">📖</span>
            使用说明
          </div>
          <div class="section-content">
            {{ selectedFunction.usage }}
          </div>
        </div>

        <!-- 参数信息 -->
        <div
          v-if="selectedFunction.params && selectedFunction.params.length > 0"
          class="detail-section"
        >
          <div class="section-label">
            <span class="section-icon">🔧</span>
            参数列表
          </div>
          <div class="section-content">
            <div class="params-list">
              <div v-for="param in selectedFunction.params" :key="param.name" class="param-item">
                <div class="param-header">
                  <span class="param-name">{{ param.name }}</span>
                  <span class="param-type">{{ param.type }}</span>
                  <span v-if="param.required" class="param-required">必需</span>
                  <span v-else class="param-optional">可选</span>
                </div>
                <div class="param-description">{{ param.description }}</div>
                <div v-if="param.default" class="param-default">
                  默认值: <code>{{ param.default }}</code>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 使用示例 -->
        <div v-if="selectedFunction.example" class="detail-section">
          <div class="section-label">
            <span class="section-icon">💡</span>
            使用示例
          </div>
          <div class="section-content code-content">
            {{ selectedFunction.example }}
          </div>
        </div>

        <!-- 插入文本 -->
        <div class="detail-section">
          <div class="section-label">
            <span class="section-icon">✨</span>
            点击插入
          </div>
          <div class="section-content code-content insert-preview">
            {{ selectedFunction.insertText || selectedFunction.insert_text }}
          </div>
        </div>
      </div>
    </div>

    <!-- 宽度调整手柄 -->
    <div class="sidebar-resizer" @mousedown="startResize" :class="{ resizing: isResizing }">
      <div class="resizer-dots">
        <div class="dot"></div>
        <div class="dot"></div>
        <div class="dot"></div>
      </div>
    </div>

    <!-- 订阅预设变量弹窗 -->
    <n-modal v-model:show="showSubscribeModal" :mask-closable="false">
      <n-card style="width: 800px" title="订阅" :bordered="false" size="huge">
        <template #header-extra>
          <n-button quaternary circle @click="showSubscribeModal = false">
            <template #icon>
              <n-icon><CloseOutline /></n-icon>
            </template>
          </n-button>
        </template>

        <!-- 搜索栏 -->
        <div class="search-bar">
          <n-input
            v-model:value="searchKeyword"
            placeholder="搜索预设变量..."
            clearable
            @update:value="searchPresets"
          >
            <template #prefix>
              <n-icon><SearchOutline /></n-icon>
            </template>
          </n-input>
        </div>

        <!-- 可用预设变量列表 -->
        <n-spin :show="presetsLoading">
          <div class="available-presets">
            <n-empty v-if="availablePresets.length === 0" description="暂无可用的预设变量" />

            <div v-else class="presets-list">
              <div class="preset-item-modal" v-for="preset in availablePresets" :key="preset.id">
                <div class="preset-content">
                  <n-checkbox
                    :checked="isPresetSelected(preset.id)"
                    @update:checked="togglePreset(preset, $event)"
                  >
                    <div class="preset-info-modal">
                      <div class="preset-name">{{ preset.name }}</div>
                      <div class="preset-description" v-if="preset.description">
                        {{ preset.description }}
                      </div>
                    </div>
                  </n-checkbox>
                </div>
              </div>
            </div>
          </div>
        </n-spin>

        <!-- 分页 -->
        <div class="pagination" v-if="totalPresets > pageSize">
          <n-pagination
            v-model:page="currentPage"
            :page-size="pageSize"
            :item-count="totalPresets"
            @update:page="loadAvailablePresets"
            show-size-picker
            :page-sizes="[10, 20, 30]"
            @update:page-size="handlePageSizeChange"
          />
        </div>

        <template #footer>
          <div class="modal-footer">
            <n-button @click="showSubscribeModal = false">取消</n-button>
            <n-button
              type="primary"
              @click="confirmSubscribe"
              :loading="subscribing"
              :disabled="selectedPresets.length === 0"
            >
              订阅 ({{ selectedPresets.length }})
            </n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
  import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
  import {
    NSpin,
    NButton,
    NTag,
    NEmpty,
    NIcon,
    NModal,
    NCard,
    NInput,
    NCheckbox,
    NPagination,
    useMessage,
  } from 'naive-ui';
  import {
    CloseOutline,
    AppsOutline,
    CodeSlash,
    CodeOutline,
    ConstructOutline,
    BuildOutline,
    SearchOutline,
    FolderOutline,
    ExtensionPuzzleOutline,
    TextOutline,
    RadioButtonOn,
    ListOutline,
    GridOutline,
    PersonOutline,
    Settings,
    AddOutline,
  } from '@vicons/ionicons5';
  import { getTemplateExpose } from '@/api/templateExpose';
  import {
    getSubscribedPresets,
    unsubscribePreset,
    getAvailablePresets,
    subscribePreset,
  } from '@/api/templateVariablePresets';

  const props = defineProps({
    templateVariables: {
      type: Array,
      default: () => [],
    },
    templateSyntaxCategories: {
      type: Array,
      default: () => [],
    },
    builtinFunctionCategories: {
      type: Array,
      default: () => [],
    },
    loadingFunctions: {
      type: Boolean,
      default: false,
    },
    quickVariables: {
      type: Array,
      default: () => [],
    },
    templateId: {
      type: [String, Number],
      required: true,
    },
  });

  const emit = defineEmits([
    'insert-syntax',
    'insert-function',
    'insert-variable',
    'insert-preset-variable',
    'update:width',
    'show-quick-design', // 快速设计模式
    'show-test-data',
  ]);

  // 状态
  const activeTab = ref('syntax');
  const sidebarWidth = ref(280);
  const isResizing = ref(false);

  // 变量定义相关状态
  const variableDefinitions = ref([]);
  const loadingVariableDefinitions = ref(false);

  // 预设变量相关状态
  const message = useMessage();
  const subscribedPresets = ref([]);
  const loadingPresets = ref(false);

  // 订阅弹窗相关状态
  const showSubscribeModal = ref(false);
  const availablePresets = ref([]);
  const presetsLoading = ref(false);
  const selectedPresets = ref([]);
  const subscribing = ref(false);
  const currentPage = ref(1);
  const pageSize = ref(20);
  const totalPresets = ref(0);
  const searchKeyword = ref('');

  // 悬浮卡片相关状态
  const functionDetailVisible = ref(false);
  const selectedFunction = ref(null);
  const functionDetailStyle = ref({});
  let hideTimer = null;
  let showTimer = null;

  // 垂直 Tabs 配置
  const variableTabs = computed(() => {
    const tabs = [
      {
        key: 'syntax',
        label: '模板语法',
        icon: CodeSlash,
        count: props.templateSyntaxCategories.reduce(
          (sum, cat) => sum + (cat.syntaxes?.length || 0),
          0
        ),
      },
      {
        key: 'functions',
        label: '内置函数',
        icon: ConstructOutline,
        count: props.builtinFunctionCategories.reduce(
          (sum, cat) => sum + (cat.functions?.length || 0),
          0
        ),
      },
      {
        key: 'builtin',
        label: '内置变量',
        icon: BuildOutline,
        count: props.quickVariables.length,
      },
      {
        key: 'custom',
        label: '用户变量',
        icon: PersonOutline,
        count: textVariables.value.length,
      },
    ];

    // 动态添加订阅的预设变量tab
    subscribedPresets.value.forEach((preset) => {
      tabs.push({
        key: `preset_${preset.presetId}`,
        label: preset.presetName,
        icon: AppsOutline,
        count: (presetVariables[`preset_${preset.presetId}`] || []).length,
        isPreset: true,
        presetId: preset.presetId,
      });
    });

    // 始终显示订阅管理tab（用于订阅和管理）
    tabs.push({
      key: 'subscription_manager',
      label: '订阅管理',
      icon: Settings,
      count: 0,
    });

    return tabs;
  });

  // 解析变量定义
  const parseVariableDefinitions = (fieldSchemaJson) => {
    const variables = [];

    try {
      const schema = JSON.parse(fieldSchemaJson);

      const parseSchema = (schemaObj, parentPath = '') => {
        if (!schemaObj || typeof schemaObj !== 'object') return;

        for (const [key, value] of Object.entries(schemaObj)) {
          if (value && typeof value === 'object') {
            const currentPath = parentPath ? `${parentPath}.${key}` : key;

            const variable = {
              id: `var_def_${currentPath}`,
              name: key,
              displayName: value.title || key,
              description: value.description || '',
              variableType: value.type || 'string',
              path: currentPath,
              insertText: value.insertText || `{{ ${currentPath} }}`,
              isRequired: value.required || false,
              defaultValue: value.default,
              level: (currentPath.match(/\./g) || []).length,
              parentPath: parentPath,
            };

            variables.push(variable);

            if (value.type === 'object' && value.properties) {
              parseSchema(value.properties, currentPath);
            }

            if (value.type === 'object_arr' && value.items && value.items.properties) {
              parseSchema(value.items.properties, currentPath);
            }
          }
        }
      };

      parseSchema(schema);
    } catch (error) {
      console.error('解析变量定义失败:', error);
    }

    return variables;
  };

  // 加载变量定义
  const loadVariableDefinitions = async () => {
    if (!props.templateId) return;

    loadingVariableDefinitions.value = true;
    try {
      const response = await getTemplateExpose({ templateId: props.templateId });

      if (
        response.data &&
        response.data.data &&
        response.data.data.templateExpose &&
        response.data.data.templateExpose.fieldSchemaJson
      ) {
        const parsedVariables = parseVariableDefinitions(
          response.data.data.templateExpose.fieldSchemaJson
        );
        variableDefinitions.value = parsedVariables;
      } else {
        variableDefinitions.value = [];
      }
    } catch (error) {
      console.error('加载变量定义失败:', error);
      variableDefinitions.value = [];
    } finally {
      loadingVariableDefinitions.value = false;
    }
  };

  // 计算属性：文本类型变量
  const textVariables = computed(() => {
    return variableDefinitions.value.filter(
      (v) =>
        v.variableType === 'text' ||
        v.variableType === 'string' ||
        v.variableType === '字符串' ||
        v.variableType === 'number' ||
        v.variableType === 'integer' ||
        v.variableType === '数字' ||
        v.variableType === 'boolean' ||
        v.variableType === '布尔值' ||
        v.variableType === 'array' ||
        v.variableType === 'list' ||
        v.variableType === '列表' ||
        v.variableType === 'object' ||
        v.variableType === '对象' ||
        v.variableType === 'object_arr' ||
        v.variableType === '对象数组' ||
        !v.variableType
    );
  });

  // 获取变量类型标签
  const getVariableTypeLabel = (type) => {
    const typeLabels = {
      string: '字符串',
      字符串: '字符串',
      text: '文本',
      文本: '文本',
      number: '数字',
      integer: '整数',
      数字: '数字',
      boolean: '布尔',
      布尔值: '布尔',
      array: '数组',
      list: '列表',
      列表: '列表',
      object: '对象',
      对象: '对象',
      object_arr: '对象数组',
      对象数组: '对象数组',
    };
    return typeLabels[type] || type || '文本';
  };

  // 获取变量类型图标
  const getVariableTypeIcon = (type) => {
    const iconMap = {
      string: TextOutline,
      text: TextOutline,
      number: RadioButtonOn,
      integer: RadioButtonOn,
      boolean: RadioButtonOn,
      array: ListOutline,
      list: ListOutline,
      object: GridOutline,
      object_arr: FolderOutline,
    };
    return iconMap[type] || TextOutline;
  };

  // 获取变量类型颜色
  const getVariableTypeColor = (type) => {
    const colorMap = {
      string: '#1890ff',
      text: '#1890ff',
      number: '#52c41a',
      integer: '#52c41a',
      boolean: '#fa8c16',
      array: '#722ed1',
      list: '#722ed1',
      object: '#eb2f96',
      object_arr: '#9254de',
    };
    return colorMap[type] || '#1890ff';
  };

  // 获取变量标签类型
  const getVariableTypeTagType = (type) => {
    const typeMap = {
      string: 'info',
      text: 'info',
      number: 'success',
      integer: 'success',
      boolean: 'warning',
      array: 'default',
      list: 'default',
      object: 'error',
      object_arr: 'warning',
    };
    return typeMap[type] || 'default';
  };

  // 事件处理
  const handleInsertSyntax = (syntax) => {
    emit('insert-syntax', syntax);
  };

  const handleInsertFunction = (func) => {
    emit('insert-function', func);
  };

  const handleInsertVariable = (variableName) => {
    emit('insert-variable', variableName);
  };

  const handleInsertVariableDefinition = (variable) => {
    const insertText = variable.insertText || `{{ ${variable.path || variable.name} }}`;
    emit('insert-preset-variable', insertText);
  };

  const handleInsertPresetVariable = (variable) => {
    emit('insert-preset-variable', variable.insertText);
  };

  // ========== 悬浮卡片功能 ==========
  const showFunctionDetail = (item, event) => {
    clearTimeout(hideTimer);

    showTimer = setTimeout(() => {
      selectedFunction.value = item;

      const rect = event.target.getBoundingClientRect();

      // 检测文件树是否显示
      const fileTreeElement = document.querySelector('.template-explorer');
      const isFileTreeVisible = fileTreeElement && fileTreeElement.offsetWidth > 0;
      const spacing = 10; // 间距

      // 如果文件树可见，显示在左侧（朝向文件树）
      // 如果文件树不可见，显示在右侧（朝向编辑器）
      if (isFileTreeVisible) {
        // 显示在左侧
        functionDetailStyle.value = {
          position: 'fixed',
          right: window.innerWidth - rect.left + spacing + 'px',
          top: rect.top + 'px',
          zIndex: 1000,
          maxWidth: '400px',
        };
      } else {
        // 显示在右侧
        functionDetailStyle.value = {
          position: 'fixed',
          left: rect.right + spacing + 'px',
          top: rect.top + 'px',
          zIndex: 1000,
          maxWidth: '400px',
        };
      }

      functionDetailVisible.value = true;
    }, 600); // 延迟600ms，减少误触发
  };

  const handleShowSyntaxDetail = (syntax, event) => {
    showFunctionDetail(syntax, event);
  };

  const handleShowFunctionDetail = (func, event) => {
    showFunctionDetail(func, event);
  };

  const handleShowVariableDetail = (variable, event) => {
    showFunctionDetail(variable, event);
  };

  const handleShowVariableDefinitionDetail = (variable, event) => {
    showFunctionDetail(variable, event);
  };

  const handleHideFunctionDetail = () => {
    clearTimeout(showTimer);

    hideTimer = setTimeout(() => {
      functionDetailVisible.value = false;
      selectedFunction.value = null;
    }, 100); // 减少延迟从200ms到100ms，让卡片更快消失
  };

  const onDetailPanelEnter = () => {
    clearTimeout(hideTimer);
  };

  // 预设变量解析
  const parseSchemaToVariables = (schemaJson, presetName) => {
    const variables = [];

    try {
      const schema = JSON.parse(schemaJson);

      const parseObject = (obj, parentPath = '') => {
        for (const [key, value] of Object.entries(obj)) {
          if (value && typeof value === 'object') {
            const currentPath = parentPath ? `${parentPath}.${key}` : key;

            const hasChildren = value.children && typeof value.children === 'object';

            variables.push({
              name: currentPath,
              displayName: value.displayName || key,
              description: value.description || '',
              insertText: value.insertText || `{{ ${currentPath} }}`,
              type: value.type || 'field',
              category: value.category || 'preset',
              isParent: hasChildren,
              level: (currentPath.match(/\./g) || []).length,
              parentPath: parentPath,
            });

            if (hasChildren) {
              parseObject(value.children, currentPath);
            }
          }
        }
      };

      parseObject(schema);
    } catch (error) {
      console.error('解析schema失败:', error);
    }

    return variables;
  };

  // 计算每个预设变量包的变量列表
  const presetVariables = computed(() => {
    const presetVars = {};

    subscribedPresets.value.forEach((preset) => {
      if (preset.schema) {
        presetVars[`preset_${preset.presetId}`] = parseSchemaToVariables(
          preset.schema,
          preset.presetName
        );
      }
    });

    return presetVars;
  });

  // 加载已订阅的预设变量
  const loadSubscribedPresets = async () => {
    if (!props.templateId) return;

    loadingPresets.value = true;
    try {
      const response = await getSubscribedPresets(props.templateId);
      subscribedPresets.value = response.data.data?.list || [];
    } catch (error) {
      console.error('加载预设变量失败:', error);
      subscribedPresets.value = [];
    } finally {
      loadingPresets.value = false;
    }
  };

  // 取消订阅预设变量
  const handleUnsubscribePreset = async (preset) => {
    try {
      await unsubscribePreset(props.templateId, preset.presetId);
      message.success('取消订阅成功');
      await loadSubscribedPresets();
    } catch (error) {
      console.error('取消订阅失败:', error);
      message.error('取消订阅失败');
    }
  };

  // 加载可用预设变量
  const loadAvailablePresets = async (page = 1) => {
    presetsLoading.value = true;
    try {
      const response = await getAvailablePresets({
        pageNum: page,
        pageSize: pageSize.value,
        keyword: searchKeyword.value,
      });

      const data = response.data.data || {};
      availablePresets.value = data.list || [];
      totalPresets.value = data.total || 0;
      currentPage.value = data.pageNum || page;
    } catch (error) {
      console.error('加载可用预设变量失败:', error);
      message.error('加载可用预设变量失败');
    } finally {
      presetsLoading.value = false;
    }
  };

  // 搜索预设变量
  let searchTimeout = null;
  const searchPresets = () => {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    searchTimeout = setTimeout(() => {
      currentPage.value = 1;
      loadAvailablePresets();
    }, 300);
  };

  // 分页处理
  const handlePageSizeChange = (newSize) => {
    pageSize.value = newSize;
    currentPage.value = 1;
    loadAvailablePresets();
  };

  // 变量选择
  const isPresetSelected = (presetId) => {
    return selectedPresets.value.includes(presetId);
  };

  const togglePreset = (preset, checked) => {
    if (checked) {
      if (!selectedPresets.value.includes(preset.id)) {
        selectedPresets.value.push(preset.id);
      }
    } else {
      const index = selectedPresets.value.indexOf(preset.id);
      if (index > -1) {
        selectedPresets.value.splice(index, 1);
      }
    }
  };

  // 确认订阅
  const confirmSubscribe = async () => {
    if (selectedPresets.value.length === 0) {
      message.warning('请选择要订阅的预设变量');
      return;
    }

    subscribing.value = true;
    try {
      await subscribePreset(props.templateId, selectedPresets.value);
      message.success('订阅成功');
      showSubscribeModal.value = false;
      selectedPresets.value = [];
      await loadSubscribedPresets();
    } catch (error) {
      console.error('订阅失败:', error);
      message.error('订阅失败');
    } finally {
      subscribing.value = false;
    }
  };

  // 宽度调整
  const minWidth = 200;
  const maxWidth = 500;
  let startX = 0;
  let startWidth = 0;

  const startResize = (e) => {
    isResizing.value = true;
    startX = e.clientX;
    startWidth = sidebarWidth.value;

    document.addEventListener('mousemove', onResize);
    document.addEventListener('mouseup', stopResize);

    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'ew-resize';
  };

  const onResize = (e) => {
    if (!isResizing.value) return;

    const deltaX = e.clientX - startX;
    const newWidth = Math.max(minWidth, Math.min(maxWidth, startWidth + deltaX));

    sidebarWidth.value = newWidth;
    emit('update:width', newWidth);
  };

  const stopResize = () => {
    isResizing.value = false;

    document.removeEventListener('mousemove', onResize);
    document.removeEventListener('mouseup', stopResize);

    document.body.style.userSelect = '';
    document.body.style.cursor = '';
  };

  // 监听模板ID变化
  watch(
    () => props.templateId,
    () => {
      if (props.templateId) {
        loadSubscribedPresets();
        loadVariableDefinitions();
      }
    },
    { immediate: true }
  );

  // 监听活动tab变化
  watch(
    () => activeTab.value,
    (newTab) => {
      if (newTab === 'subscription_manager' && props.templateId) {
        loadSubscribedPresets();
      }
      if (newTab === 'custom' && props.templateId) {
        loadVariableDefinitions();
      }
    }
  );

  // 监听弹窗打开
  watch(
    () => showSubscribeModal.value,
    (show) => {
      if (show) {
        selectedPresets.value = [];
        currentPage.value = 1;
        searchKeyword.value = '';
        loadAvailablePresets();
      }
    }
  );

  // 清理
  onUnmounted(() => {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }
    if (hideTimer) {
      clearTimeout(hideTimer);
    }
    if (showTimer) {
      clearTimeout(showTimer);
    }
    document.removeEventListener('mousemove', onResize);
    document.removeEventListener('mouseup', stopResize);
  });
</script>

<style scoped>
  /* 侧边栏容器 */
  .variable-sidebar {
    position: relative;
    background: #fff;
    border-right: 1px solid #e0e0e0;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* 头部 */
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fafafa;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .header-title {
    font-size: 14px;
    font-weight: 600;
    color: #333;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .action-btn {
    padding: 4px 8px !important;
    min-width: auto;
  }

  .action-btn:hover {
    background: #f9f0ff !important;
    color: #722ed1 !important;
  }

  /* 内容区域 */
  .sidebar-content {
    flex: 1;
    display: flex;
    overflow: hidden;
    padding: 0;
  }

  /* 垂直 Tabs - 左侧固定宽度 */
  .vertical-tabs {
    display: flex;
    flex-direction: column;
    width: 80px;
    min-width: 80px;
    border-right: 1px solid #e0e0e0;
    background: #fafafa;
    overflow-y: auto;
    scrollbar-width: none !important; /* Firefox */
    -ms-overflow-style: none !important; /* IE/Edge */
  }

  .vertical-tabs::-webkit-scrollbar {
    width: 0 !important;
    height: 0 !important;
  }

  .tab-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    min-height: 48px;
    padding: 8px 6px;
    cursor: pointer;
    user-select: none;
    transition: all 0.2s;
    color: #666;
    border-left: 2px solid transparent;
    text-align: center;
    position: relative;
  }

  .tab-item:hover {
    background: rgba(114, 46, 209, 0.05);
    color: #722ed1;
  }

  .tab-item.active {
    background: #fff;
    color: #722ed1;
    border-left-color: #722ed1;
    font-weight: 500;
  }

  /* 预设变量tab样式 */
  .tab-item.preset-tab {
    position: relative;
  }

  .tab-item.preset-tab .tab-badge {
    position: absolute;
    top: -2px;
    right: -2px;
    background: #722ed1;
    color: white;
    font-size: 10px;
    padding: 1px 4px;
    border-radius: 8px;
    line-height: 1.2;
    font-weight: 500;
    transform: scale(0.8);
    transform-origin: top right;
  }

  .tab-label {
    font-size: 11px;
    line-height: 1.3;
  }

  /* Tab 内容区域 - 右侧自适应 */
  .tab-pane {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    min-width: 0;
    scrollbar-width: none !important; /* Firefox */
    -ms-overflow-style: none !important; /* IE/Edge */
  }

  .tab-pane::-webkit-scrollbar {
    width: 0 !important;
    height: 0 !important;
  }

  /* 变量分类 */
  .variable-category {
    margin-bottom: 16px;
  }

  .category-divider {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    margin: 8px 0 8px 0;
    background: linear-gradient(180deg, #f8f9fa 0%, #f0f0f0 100%);
    border-left: 3px solid #18a058;
    border-radius: 4px;
  }

  .category-divider.preset-category {
    border-left-color: #722ed1;
  }

  .category-name {
    font-size: 12px;
    font-weight: 600;
    color: #333;
  }

  .category-count {
    font-size: 11px;
    color: #666;
    background: #e8e8e8;
    padding: 2px 6px;
    border-radius: 10px;
  }

  /* 变量列表 */
  .variable-items {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .variable-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .variable-item:hover {
    background: #f0f9ff;
    border-left: 2px solid #18a058;
  }

  .variable-info {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
  }

  .variable-name {
    font-size: 12px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    color: #333;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* 预设变量特殊样式 */
  .preset-item.parent-variable {
    background: #f9f0ff;
    border-left: 2px solid #722ed1;
    font-weight: 500;
  }

  .preset-item.child-variable {
    opacity: 0.9;
  }

  /* 订阅管理项样式 */
  .preset-subscription-item {
    background: #fafafa;
  }

  .preset-subscription-item:hover {
    background: #f0f9ff;
  }

  .preset-subscription-item .variable-info {
    cursor: pointer;
    flex: 1;
  }

  .subscribe-tip {
    margin-top: 12px;
    padding: 0 4px;
  }

  /* 空状态和加载状态 */
  .empty-state,
  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    color: #999;
    font-size: 13px;
    flex-direction: column;
    gap: 8px;
  }

  /* 调整手柄 */
  .sidebar-resizer {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 6px;
    background: transparent;
    cursor: ew-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }

  .sidebar-resizer:hover {
    background: rgba(24, 160, 88, 0.05);
  }

  .sidebar-resizer.resizing {
    background: rgba(24, 160, 88, 0.1);
  }

  .resizer-dots {
    display: flex;
    gap: 2px;
  }

  .resizer-dots .dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: #ccc;
  }

  .sidebar-resizer:hover .dot {
    background: #18a058;
  }

  /* 订阅弹窗样式 */
  .search-bar {
    margin-bottom: 16px;
  }

  .available-presets {
    max-height: 400px;
    overflow-y: auto;
  }

  .preset-item-modal {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 12px;
  }

  .preset-info-modal .preset-name {
    margin: 0 0 8px 0;
    color: #333;
    font-weight: 500;
    font-size: 14px;
  }

  .preset-info-modal .preset-description {
    margin: 0;
    color: #666;
    font-size: 13px;
  }

  .pagination {
    margin-top: 16px;
    text-align: center;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    align-items: center;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid #f0f0f0;
  }

  /* ========== 悬浮卡片样式 ========== */
  .function-detail-card {
    background: #fff;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    padding: 16px;
    max-width: 400px;
    font-size: 13px;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid #f0f0f0;
  }

  .detail-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
    font-size: 14px;
    color: #333;
  }

  .function-icon {
    font-size: 16px;
  }

  .detail-type {
    font-size: 11px;
    padding: 2px 8px;
    background: #f0f0f0;
    border-radius: 4px;
    color: #666;
    font-weight: 500;
  }

  .detail-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .detail-description {
    color: #666;
    line-height: 1.6;
    font-size: 13px;
  }

  .detail-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .section-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 500;
    color: #333;
    font-size: 12px;
  }

  .section-icon {
    font-size: 14px;
  }

  .section-content {
    padding: 8px 12px;
    background: #f8f9fa;
    border-radius: 4px;
    font-size: 12px;
    line-height: 1.5;
    color: #555;
  }

  .code-content {
    font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
    background: #f0f0f0;
    color: #d73a49;
    word-break: break-all;
  }

  .insert-preview {
    color: #18a058;
    font-weight: 500;
  }

  .params-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .param-item {
    padding: 8px;
    background: #fff;
    border: 1px solid #e0e0e0;
    border-radius: 4px;
  }

  .param-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .param-name {
    font-weight: 600;
    color: #333;
    font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
    font-size: 12px;
  }

  .param-type {
    font-size: 11px;
    padding: 2px 6px;
    background: #e8f4f8;
    color: #0066cc;
    border-radius: 3px;
    font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
  }

  .param-required {
    font-size: 10px;
    padding: 2px 6px;
    background: #fff1f0;
    color: #ff4d4f;
    border-radius: 3px;
    font-weight: 500;
  }

  .param-optional {
    font-size: 10px;
    padding: 2px 6px;
    background: #f6ffed;
    color: #52c41a;
    border-radius: 3px;
    font-weight: 500;
  }

  .param-description {
    color: #666;
    font-size: 12px;
    line-height: 1.4;
    margin: 4px 0;
  }

  .param-default {
    font-size: 11px;
    color: #888;
    margin-top: 4px;
  }

  .param-default code {
    background: #f0f0f0;
    padding: 2px 4px;
    border-radius: 3px;
    font-family: 'Monaco', 'Consolas', 'Courier New', monospace;
    color: #d73a49;
  }
</style>
