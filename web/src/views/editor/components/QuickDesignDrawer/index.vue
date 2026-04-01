<template>
  <n-drawer
    v-model:show="drawerVisible"
    :width="'100%'"
    placement="right"
    :trap-focus="false"
    :block-scroll="false"
    :native-scrollbar="false"
  >
    <div class="drawer-container">
      <!-- Header -->
      <StudioHeader
        v-model:edit-mode="editMode"
        v-model:show-design="showDesign"
        v-model:show-schema="showSchema"
        v-model:show-form="showForm"
        :saving="saving"
        @show-test-data="handleShowTestData"
        @show-variable-analysis="handleShowVariableAnalysis"
        @save="handleSave"
        @refresh="handleRefresh"
        @close="handleCloseDrawer"
      />

      <!-- 主体内容区域 - 三栏布局 -->
      <div class="main-content-wrapper">
        <div class="three-column-layout">
          <!-- 左栏：设计区域 -->
          <div
            v-show="showDesign"
            class="layout-column design-column"
            :style="{ width: calculateColumnWidth() }"
          >
            <!-- 设计模式 -->
            <div v-if="editMode === 'design'" class="design-canvas-container">
              <!-- 左侧：组件库 -->
              <ComponentLibrary
                :categories="COMPONENT_CATEGORIES"
                @add-component="handleAddComponent"
                @drag-start="handleDragStart"
              />

              <!-- 中间：设计画布 -->
              <div class="canvas-wrapper">
                <!-- 面包屑导航 -->
                <div v-if="navigationPath.length > 0" class="breadcrumb-nav">
                  <n-breadcrumb>
                    <n-breadcrumb-item @click="handleNavigateToRoot">
                      <template #separator>
                        <n-icon size="12"><ChevronForwardOutline /></n-icon>
                      </template>
                      变量（根层级）
                    </n-breadcrumb-item>
                    <n-breadcrumb-item
                      v-for="(item, index) in navigationPath"
                      :key="item.id"
                      @click="handleNavigateToLevel(index)"
                    >
                      <template #separator>
                        <n-icon size="12"><ChevronForwardOutline /></n-icon>
                      </template>
                      {{ item.fieldName || item.title }}（第{{ index + 1 }}层）
                    </n-breadcrumb-item>
                  </n-breadcrumb>
                  <!-- 层级提示 -->
                  <n-text
                    v-if="navigationPath.length >= WARNING_LEVEL"
                    :type="navigationPath.length >= MAX_NESTING_LEVEL ? 'error' : 'warning'"
                    depth="3"
                    style="margin-left: 12px; font-size: 12px"
                  >
                    {{
                      navigationPath.length >= MAX_NESTING_LEVEL
                        ? '已达最大层级'
                        : `${navigationPath.length + 1}/${MAX_NESTING_LEVEL}层`
                    }}
                  </n-text>
                </div>

                <DesignCanvas
                  :components="currentLevelComponents"
                  :selected-component-id="selectedComponentId"
                  :expanded-components="expandedComponents"
                  :is-container-component="isContainerComponent"
                  :get-children-count="getChildrenCount"
                  @drop="handleDrop"
                  @select-component="selectComponent"
                  @remove-component="handleRemoveComponent"
                  @toggle-expand="toggleComponentExpansion"
                  @select-child="selectComponent"
                  @remove-child="handleRemoveChildFromCanvas"
                  @enter-component="handleEnterComponent"
                />
              </div>

              <!-- 右侧：属性面板 -->
              <PropertyPanel
                :key="`prop-design-${effectiveSelectedComponent?.id || 'none'}-${
                  effectiveSelectedComponent?.type || 'unknown'
                }`"
                :component="effectiveSelectedComponent"
                mode="design"
                @update:component="handlePropertyUpdate"
              />
            </div>

            <!-- 变量树模式 -->
            <div v-else-if="editMode === 'tree'" class="tree-editor-container">
              <!-- 左侧：变量树 -->
              <VariableTree
                :components="components"
                :selected-component-id="selectedComponentId"
                :expanded-keys="expandedKeys"
                @update:selected-component-id="selectedComponentId = $event"
                @update:expanded-keys="expandedKeys = $event"
                @add-variable="handleAddVariable"
                @delete-variable="handleDeleteVariable"
                @rename-variable="handleRenameVariable"
              />

              <!-- 右侧：属性面板（共享） -->
              <PropertyPanel
                :key="`prop-tree-${effectiveSelectedComponent?.id || 'none'}-${
                  effectiveSelectedComponent?.type || 'unknown'
                }`"
                :component="effectiveSelectedComponent"
                mode="tree"
                @update:component="handlePropertyUpdate"
              />
            </div>
          </div>

          <!-- 中栏：Schema 预览 -->
          <div
            v-show="showSchema"
            class="layout-column schema-column"
            :style="{ width: calculateColumnWidth() }"
          >
            <SchemaEditor
              ref="schemaEditorRef"
              v-if="showSchema"
              :schema="formattedSchema"
              :show="showSchema"
              :template-id="templateId"
              @sync="handleSyncSchema"
              @reset="handleReset"
            />
          </div>

          <!-- 右栏：表单预览 -->
          <div
            v-show="showForm"
            class="layout-column form-column"
            :style="{ width: calculateColumnWidth() }"
          >
            <div class="form-preview-container">
              <div class="preview-header">
                <n-text strong>表单预览</n-text>
              </div>
              <div class="preview-content">
                <FormPreview
                  v-if="showForm"
                  :schema="currentSchemaForPreview"
                  @change="handleFormChange"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <StudioFooter :component-count="componentCount" :has-unsaved-changes="hasUnsavedChanges" />

      <!-- 测试数据模态框 -->
      <TestDataModal
        v-model:show="showTestDataModal"
        :template-id="templateId"
        :schema="currentSchemaForPreview"
        @test-data-updated="handleTestDataUpdated"
      />

      <!-- 变量分析模态框 -->
      <VariableAnalysisModal
        v-model:show="showVariableAnalysisModal"
        :template-id="templateId"
        :components="components"
        @add-components="handleAddDetectedComponents"
        @remove-components="handleRemoveUnusedComponents"
      />
    </div>
  </n-drawer>
</template>

<script setup>
  import { ref, computed, watch, onMounted, provide, readonly, toRef, nextTick } from 'vue';
  import { NDrawer, NText, NBreadcrumb, NBreadcrumbItem, NIcon } from 'naive-ui';
  import { useMessage } from 'naive-ui';
  import { ChevronForwardOutline } from '@vicons/ionicons5';
  import { useSchemaStore } from './composables/useSchemaStore';
  import { useComponentManager } from './composables/useComponentManager';
  import {
    COMPONENT_CATEGORIES,
    generateFieldName,
    generateDefaultSchema,
  } from './utils/componentTemplates';
  import StudioHeader from './components/StudioHeader.vue';
  import StudioFooter from './components/StudioFooter.vue';
  import ComponentLibrary from './components/ComponentLibrary.vue';
  import DesignCanvas from './components/DesignCanvas.vue';
  import PropertyPanel from './components/PropertyPanel.vue';
  import VariableTree from './components/VariableTree.vue';
  import SchemaEditor from './components/SchemaEditor.vue';
  import FormPreview from './components/FormPreview.vue';
  import TestDataModal from './components/TestDataModal.vue';
  import VariableAnalysisModal from './components/VariableAnalysisModal.vue';

  /**
   * QuickDesignDrawer 主容器组件
   * 负责整合所有子组件，管理全局状态
   */

  // Props
  const props = defineProps({
    show: Boolean,
    templateId: [String, Number],
  });

  // Emits
  const emit = defineEmits(['update:show', 'save', 'test-data-updated']);

  const message = useMessage();

  // ========== 状态管理 ==========

  /**
   * Schema Store
   * 注意：需要传递 templateId，否则无法加载数据
   */
  const schemaStore = useSchemaStore(props.templateId);

  /**
   * 组件管理
   */
  const componentManager = useComponentManager(schemaStore);

  // 从 componentManager 解构出需要的状态和方法
  const {
    components,
    selectedComponentId,
    selectedComponent,
    componentCount,
    hasUnsavedChanges,
    expandedComponents,
    formattedSchema,
    currentSchemaForPreview,
    addComponent,
    removeComponent,
    selectComponent,
    updateComponentFieldName,
    resetComponents,
    componentsToSchema,
    isContainerComponent,
    getChildrenCount,
    toggleComponentExpansion,
    addChildToContainer,
    removeChildFromContainer,
  } = componentManager;

  // UI 状态
  const drawerVisible = computed({
    get: () => props.show,
    set: (val) => emit('update:show', val),
  });

  const editMode = ref('design'); // 'design' | 'tree'
  const expandedKeys = ref([]); // 变量树展开的节点
  const showDesign = ref(true);
  const showSchema = ref(true);
  const showForm = ref(true);
  const saving = ref(false);
  const isLoading = ref(false); // 防止加载时触发无限循环
  let isUpdating = false; // 非响应式标志，防止属性更新时触发无限循环
  let lastTypeUpdateTime = 0; // 记录上次类型更新时间
  let isSyncingOrSaving = false; // 防止同步和保存期间触发watch

  // 导航路径（用于进入复杂组件内部）
  const navigationPath = ref([]); // 存储导航路径，每个元素包含 { id, fieldName, title, component }

  // 嵌套层级限制
  const MAX_NESTING_LEVEL = 5; // 最大嵌套层级
  const WARNING_LEVEL = 3; // 警告层级

  // 模态框状态
  const showTestDataModal = ref(false);
  const showVariableAnalysisModal = ref(false);

  // 子组件引用
  const schemaEditorRef = ref(null);

  // ========== 计算属性 ==========

  /**
   * 计算可见列的数量
   */
  const visibleColumnCount = computed(() => {
    let count = 0;
    if (showDesign.value) count++;
    if (showSchema.value) count++;
    if (showForm.value) count++;
    return count;
  });

  /**
   * 计算每列的宽度
   */
  const calculateColumnWidth = () => {
    const count = visibleColumnCount.value;
    if (count === 0) return '100%';
    if (count === 1) return '100%';
    if (count === 2) return '50%';
    return '33.33%';
  };

  /**
   * 计算当前层级的组件列表
   * 如果在根层级，返回 components
   * 如果在复杂组件内部，返回该组件的子字段
   */
  const currentLevelComponents = computed(() => {
    if (navigationPath.value.length === 0) {
      // 根层级，返回所有组件
      return components.value;
    }

    // 在复杂组件内部
    const currentComponent = navigationPath.value[navigationPath.value.length - 1].component;
    if (!currentComponent) return [];

    // 获取子字段
    const schema = currentComponent.schema;
    if (schema.type === 'object' && schema.properties) {
      // 对象类型，将 properties 转换为组件数组
      return Object.entries(schema.properties).map(([fieldName, fieldSchema]) => ({
        id: `${currentComponent.id}_${fieldName}`,
        type: fieldSchema.type || 'string',
        fieldName,
        schema: fieldSchema,
        isNested: true,
        parentId: currentComponent.id,
      }));
    } else if (schema.type === 'object_arr' && schema.items?.properties) {
      // 对象数组类型，将 items.properties 转换为组件数组
      return Object.entries(schema.items.properties).map(([fieldName, fieldSchema]) => ({
        id: `${currentComponent.id}_items_${fieldName}`,
        type: fieldSchema.type || 'string',
        fieldName,
        schema: fieldSchema,
        isNested: true,
        parentId: currentComponent.id,
      }));
    }

    return [];
  });

  /**
   * 计算当前选中的组件（支持嵌套组件）
   * 如果在根层级，使用 selectedComponent
   * 如果在复杂组件内部，从 currentLevelComponents 中查找
   */
  const effectiveSelectedComponent = computed(() => {
    if (!selectedComponentId.value) {
      return null;
    }

    // 如果在根层级，直接使用 selectedComponent
    if (navigationPath.value.length === 0) {
      return selectedComponent.value;
    }

    // 在复杂组件内部，从 currentLevelComponents 中查找
    return currentLevelComponents.value.find((c) => c.id === selectedComponentId.value) || null;
  });

  // ========== Provide 上下文 ==========

  /**
   * 提供给子组件的共享上下文
   */
  provide('studioContext', {
    // 只读数据
    templateId: readonly(toRef(props, 'templateId')),
    editMode: readonly(editMode),
    showDesign: readonly(showDesign),
    showSchema: readonly(showSchema),
    showForm: readonly(showForm),

    // 组件数据
    components: readonly(components),
    selectedComponentId: readonly(selectedComponentId),

    // 操作方法
    addComponent,
    removeComponent,
    selectComponent,
    updateComponent: (componentId, updates) => {
      if (updates.field === 'fieldName') {
        updateComponentFieldName(componentId, updates.value);
      } else if (updates.field === 'schema') {
        componentManager.updateComponentSchema(componentId, updates.value);
      }
      saveDraft();
    },
  });

  provide('schemaStore', schemaStore);

  // ========== 事件处理 ==========

  /**
   * 从组件库添加组件
   */
  const handleAddComponent = (componentTemplate) => {
    // 检查当前是否在复杂组件内部
    if (navigationPath.value.length > 0) {
      // 在复杂组件内部，需要添加子字段
      const parentComponent = navigationPath.value[navigationPath.value.length - 1].component;
      const schema = parentComponent.schema;

      // 收集已存在的字段名（避免重复）
      const existingFields = {};
      if (schema.type === 'object' && schema.properties) {
        Object.keys(schema.properties).forEach((key) => {
          existingFields[key] = true;
        });
      } else if (schema.type === 'object_arr' && schema.items?.properties) {
        Object.keys(schema.items.properties).forEach((key) => {
          existingFields[key] = true;
        });
      }

      // 生成唯一的字段名
      const fieldName = generateFieldName(componentTemplate.type, existingFields);

      // 创建子字段
      const newField = {
        id: `${parentComponent.id}_${fieldName}`,
        type: componentTemplate.type,
        fieldName,
        schema: {
          type: componentTemplate.type,
          title: fieldName,
          description: '',
          ...componentTemplate.defaultSchema,
        },
        isNested: true,
        parentId: parentComponent.id,
      };

      // 根据父组件类型添加到相应的位置
      if (schema.type === 'object') {
        if (!schema.properties) {
          schema.properties = {};
        }
        schema.properties[fieldName] = newField.schema;
      } else if (schema.type === 'object_arr') {
        if (!schema.items) {
          schema.items = { type: 'object', properties: {} };
        }
        if (!schema.items.properties) {
          schema.items.properties = {};
        }
        schema.items.properties[fieldName] = newField.schema;
      }

      // 触发父组件的 schema 更新，使其重新渲染
      const parentIndex = components.value.findIndex((c) => c.id === parentComponent.id);
      if (parentIndex > -1) {
        components.value[parentIndex] = { ...parentComponent };
      }

      message.success(
        `已添加字段"${fieldName}"到 ${parentComponent.schema.title || parentComponent.fieldName}`
      );
    } else {
      // 在根层级，正常添加组件
      addComponent(componentTemplate);
    }

    saveDraft();
  };

  /**
   * 拖拽开始
   */
  const handleDragStart = (component, event) => {
    event.dataTransfer.effectAllowed = 'copy';
    event.dataTransfer.setData('component', JSON.stringify(component));
  };

  /**
   * 移除组件（处理根层级和复杂组件内部的删除）
   */
  const handleRemoveComponent = (indexOrComponent) => {
    // 检查是否在复杂组件内部
    if (navigationPath.value.length > 0) {
      const parentComponent = navigationPath.value[navigationPath.value.length - 1].component;
      const schema = parentComponent.schema;

      // 获取要删除的组件
      let componentToDelete;
      if (typeof indexOrComponent === 'number') {
        componentToDelete = currentLevelComponents.value[indexOrComponent];
      } else {
        componentToDelete = indexOrComponent;
      }

      if (!componentToDelete) return;

      const fieldName = componentToDelete.fieldName;

      // 从父组件的 schema 中删除该字段
      if (schema.type === 'object' && schema.properties) {
        delete schema.properties[fieldName];
      } else if (schema.type === 'object_arr' && schema.items?.properties) {
        delete schema.items.properties[fieldName];
      }

      // 触发父组件的更新
      const parentIndex = components.value.findIndex((c) => c.id === parentComponent.id);
      if (parentIndex > -1) {
        // 创建新的父组件对象以触发响应式更新
        const updatedParent = { ...parentComponent };
        components.value[parentIndex] = updatedParent;

        // 同步更新 navigationPath 中的组件引用
        const navIndex = navigationPath.value.length - 1;
        if (navIndex >= 0) {
          navigationPath.value[navIndex] = {
            ...navigationPath.value[navIndex],
            component: updatedParent,
          };
        }
      }

      // 清空选中状态
      selectedComponentId.value = null;

      message.success(`已删除字段"${fieldName}"`);
    } else {
      // 在根层级，正常删除
      if (typeof indexOrComponent === 'number') {
        removeComponent(indexOrComponent);
      } else {
        const index = components.value.findIndex((c) => c.id === indexOrComponent.id);
        if (index > -1) {
          removeComponent(index);
        }
      }
    }

    saveDraft();
  };

  /**
   * 拖放处理
   */
  const handleDrop = (event) => {
    event.preventDefault();

    // 检查是否在根层级
    if (navigationPath.value.length === 0) {
      // 检查拖拽目标是否在复杂组件的子字段区域内
      const target = event.target;
      const containerChildren = target.closest('.container-children');

      if (containerChildren) {
        // 找到对应的复杂组件
        const componentWrapper = target.closest('.canvas-component-wrapper');
        if (componentWrapper) {
          const componentId = componentWrapper
            .querySelector('.canvas-component')
            ?.getAttribute('data-component-id');
          const component = components.value.find((c) => c.id === componentId);

          if (component) {
            const schema = component.schema;
            const isComplex = schema.type === 'object' || schema.type === 'object_arr';

            if (isComplex) {
              message.warning('请右键点击复杂组件，选择"进入内部编辑"后才能添加子字段');
              return;
            }
          }
        }
      }
    }

    const componentData = event.dataTransfer.getData('component');
    if (componentData) {
      try {
        const component = JSON.parse(componentData);
        handleAddComponent(component);
      } catch (error) {
        console.error('解析拖拽数据失败:', error);
      }
    }
  };

  /**
   * 从画布移除子组件
   */
  const handleRemoveChildFromCanvas = ({ container, childId }) => {
    removeChildFromContainer(container, childId);
    saveDraft();
  };

  /**
   * 属性更新
   */
  const handlePropertyUpdate = ({ field, value }) => {
    // 使用 effectiveSelectedComponent 支持嵌套组件
    const currentComponent = effectiveSelectedComponent.value;
    if (!currentComponent) return;

    // 防止在加载期间更新
    if (isLoading.value) {
      return;
    }

    // 防止并发更新导致的无限循环
    if (isUpdating) {
      return;
    }

    // 处理嵌套组件的字段名更新
    if (field === 'fieldName') {
      if (currentComponent.isNested) {
        // 嵌套组件：需要更新父组件 schema 中的键名
        const parentComponent = navigationPath.value[navigationPath.value.length - 1].component;
        const oldFieldName = currentComponent.fieldName;
        const schema = parentComponent.schema;

        // 获取旧的 schema 数据
        let oldSchemaData;
        let propertiesObj;
        if (schema.type === 'object' && schema.properties) {
          propertiesObj = schema.properties;
        } else if (schema.type === 'object_arr' && schema.items?.properties) {
          propertiesObj = schema.items.properties;
        }

        if (propertiesObj && propertiesObj[oldFieldName]) {
          oldSchemaData = propertiesObj[oldFieldName];
          delete propertiesObj[oldFieldName];
          propertiesObj[value] = oldSchemaData;
        }

        // 更新当前组件的 fieldName（用于下次更新）
        currentComponent.fieldName = value;

        // 触发父组件更新
        const parentIndex = components.value.findIndex((c) => c.id === parentComponent.id);
        if (parentIndex > -1) {
          components.value[parentIndex] = { ...parentComponent };
        }

        saveDraft();
      } else {
        // 非嵌套组件：使用原有逻辑
        updateComponentFieldName(currentComponent.id, value);
      }
      return;
    }

    // 处理类型更新
    if (field === 'type') {
      const currentType = currentComponent.type;
      if (currentType === value) {
        return;
      }

      isUpdating = true;
      lastTypeUpdateTime = Date.now();

      if (currentComponent.isNested) {
        // 嵌套组件：直接更新 schema.type
        currentComponent.type = value;
        currentComponent.schema.type = value;

        // 更新父组件以触发重新渲染
        const parentComponent = navigationPath.value[navigationPath.value.length - 1].component;
        const parentIndex = components.value.findIndex((c) => c.id === parentComponent.id);
        if (parentIndex > -1) {
          components.value[parentIndex] = { ...parentComponent };
        }
      } else {
        // 非嵌套组件：使用原有逻辑
        componentManager.updateComponentType(currentComponent.id, value);

        const newSchema = generateDefaultSchema(value, currentComponent.fieldName);
        newSchema.title = currentComponent.schema?.title || currentComponent.fieldName;
        newSchema.description = currentComponent.schema?.description || '';

        componentManager.updateComponentSchema(currentComponent.id, newSchema);
      }

      setTimeout(() => {
        isUpdating = false;
        saveDraft();
      }, 500);

      return;
    }

    // 处理 schema 更新
    if (field === 'schema') {
      if (currentComponent.isNested) {
        // 嵌套组件：直接更新 schema 对象
        Object.assign(currentComponent.schema, value);

        // 触发父组件更新
        const parentComponent = navigationPath.value[navigationPath.value.length - 1].component;
        const parentIndex = components.value.findIndex((c) => c.id === parentComponent.id);
        if (parentIndex > -1) {
          components.value[parentIndex] = { ...parentComponent };
        }

        saveDraft();
      } else {
        // 非嵌套组件：使用原有逻辑
        componentManager.updateComponentSchema(currentComponent.id, value);
      }
    }
  };

  /**
   * 添加变量（用于变量树模式）
   * @param {Object} payload - { parentId, fieldName, type }
   */
  const handleAddVariable = (payload) => {
    const { parentId, fieldName, type = 'string' } = payload;

    const schema = generateDefaultSchema(type, fieldName);
    schema.title = fieldName;
    schema.description = `${fieldName} 变量`;

    const newComponent = {
      id: `comp_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
      fieldName,
      type,
      schema,
      children: [],
      template: COMPONENT_CATEGORIES[0].components.find((c) => c.type === type),
    };

    if (parentId) {
      // 添加子变量
      const parent = findComponentById(components.value, parentId);
      if (parent) {
        // 使用 addChildToContainer 确保同步更新 schema
        componentManager.addChildToContainer(parent, newComponent);

        selectedComponentId.value = newComponent.id;
        saveDraft();
        message.success(`已添加子变量: ${fieldName}`);
      } else {
        message.error('找不到父级变量');
      }
    } else {
      // 添加根变量
      components.value.push(newComponent);
      selectedComponentId.value = newComponent.id;
      saveDraft();
      message.success(`已添加变量: ${fieldName}`);
    }
  };

  /**
   * 删除变量
   * @param {String} componentId - 组件ID
   */
  const handleDeleteVariable = (componentId) => {
    const removeComponentFromTree = (list, id) => {
      const index = list.findIndex((c) => c.id === id);
      if (index > -1) {
        list.splice(index, 1);
        return true;
      }
      for (const comp of list) {
        if (comp.children && comp.children.length > 0) {
          if (removeComponentFromTree(comp.children, id)) {
            return true;
          }
        }
      }
      return false;
    };

    if (removeComponentFromTree(components.value, componentId)) {
      saveDraft();
      message.success('变量已删除');
    } else {
      message.error('删除变量失败');
    }
  };

  /**
   * 重命名变量
   * @param {Object} payload - { id, newName }
   */
  const handleRenameVariable = (payload) => {
    const { id, newName } = payload;

    const updateComponentName = (list, targetId, newName) => {
      for (const comp of list) {
        if (comp.id === targetId) {
          comp.fieldName = newName;
          if (comp.schema) {
            comp.schema.title = newName;
          }
          return true;
        }
        if (comp.children && comp.children.length > 0) {
          if (updateComponentName(comp.children, targetId, newName)) {
            return true;
          }
        }
      }
      return false;
    };

    if (updateComponentName(components.value, id, newName)) {
      saveDraft();
      message.success(`变量已重命名为: ${newName}`);
    } else {
      message.error('重命名失败');
    }
  };

  /**
   * 根据ID查找组件
   */
  const findComponentById = (list, id) => {
    for (const comp of list) {
      if (comp.id === id) {
        return comp;
      }
      if (comp.children && comp.children.length > 0) {
        const found = findComponentById(comp.children, id);
        if (found) return found;
      }
    }
    return null;
  };
  /**
   * 同步 Schema 到设计画布
   */
  const handleSyncSchema = (schema) => {
    try {
      isSyncingOrSaving = true;
      componentManager.schemaToComponents(schema);
      message.success('Schema已同步到设计画布');

      // 同步完成后手动保存草稿（不通过saveDraft函数，避免检查）
      const draftSchema = componentsToSchema();
      schemaStore.saveDraft('quick', draftSchema);

      // 延迟恢复标志
      nextTick(() => {
        isSyncingOrSaving = false;
      });
    } catch (error) {
      isSyncingOrSaving = false;
      message.error('同步失败: ' + error.message);
    }
  };

  /**
   * 复制 Schema
   */
  const handleCopySchema = async () => {
    try {
      await navigator.clipboard.writeText(formattedSchema.value);
      message.success('Schema已复制到剪贴板');
    } catch (error) {
      message.error('复制失败');
    }
  };

  /**
   * 重置
   */
  const handleReset = () => {
    resetComponents();
    saveDraft();
    message.info('已重置');
  };

  /**
   * 保存
   */
  const handleSave = async () => {
    if (componentCount.value === 0) {
      message.warning('请先添加组件');
      return;
    }

    // 设置保护标志，防止保存过程中触发watch
    isSyncingOrSaving = true;

    saving.value = true;
    try {
      const schema = componentsToSchema();
      emit('save', schema);
      message.success('保存成功');
    } catch (error) {
      message.error('保存失败: ' + error.message);
    } finally {
      saving.value = false;

      // 延迟1秒后恢复标志，确保所有异步操作完成
      setTimeout(() => {
        isSyncingOrSaving = false;
      }, 1000);
    }
  };

  /**
   * 表单数据变化
   */
  const handleFormChange = (formData) => {
    // 未来可以扩展为实际的测试数据功能
  };

  /**
   * 显示测试数据模态框
   */
  const handleShowTestData = () => {
    showTestDataModal.value = true;
  };

  /**
   * 显示变量分析模态框
   */
  const handleShowVariableAnalysis = () => {
    showVariableAnalysisModal.value = true;
  };

  /**
   * 刷新变量（清除本地缓存并重新加载）
   */
  const handleRefresh = async () => {
    if (!props.templateId) {
      message.warning('无法刷新：缺少模板ID');
      return;
    }

    // 显示加载提示并保存实例
    const loadingMessage = message.loading('正在刷新变量...', { duration: 0 });

    try {
      // 清除本地草稿缓存
      schemaStore.clearDrafts();

      // 重新从服务器加载最新数据
      await schemaStore.loadServerSchema();

      // 清空当前组件列表
      resetComponents();

      // 重新加载变量
      await loadVariables();

      // 销毁加载提示
      loadingMessage.destroy();

      message.success('变量已刷新，已加载最新数据');
    } catch (error) {
      // 销毁加载提示
      loadingMessage.destroy();

      console.error('刷新变量失败:', error);
      message.error('刷新失败: ' + error.message);
    }
  };

  /**
   * 进入复杂组件内部
   */
  const handleEnterComponent = (component) => {
    if (!component) return;

    // 只能进入 object 或 object_arr 类型的组件
    const schema = component.schema;
    if (schema.type !== 'object' && schema.type !== 'object_arr') {
      message.warning('只有对象和对象数组类型的组件才能进入内部编辑');
      return;
    }

    // 检查当前层级深度
    const currentLevel = navigationPath.value.length;

    // 达到最大层级限制
    if (currentLevel >= MAX_NESTING_LEVEL) {
      message.error(`已达到最大嵌套层级限制（${MAX_NESTING_LEVEL}层），请简化数据结构`);
      return;
    }

    // 达到警告层级，显示提示
    if (currentLevel >= WARNING_LEVEL) {
      message.warning(`当前已达到${currentLevel + 1}层嵌套，建议简化数据结构以提升可维护性`);
    }

    // 添加到导航路径
    navigationPath.value.push({
      id: component.id,
      fieldName: component.fieldName,
      title: component.schema.title,
      component: component,
    });

    // 清空当前选中的组件
    selectedComponentId.value = null;

    message.success(`已进入 ${component.schema.title || component.fieldName} 的内部`);
  };

  /**
   * 返回到根层级
   */
  const handleNavigateToRoot = () => {
    navigationPath.value = [];
    selectedComponentId.value = null;
  };

  /**
   * 返回到指定层级
   */
  const handleNavigateToLevel = (index) => {
    // 截断到指定层级（保留 index + 1 个元素，因为 index 是从 0 开始的）
    navigationPath.value = navigationPath.value.slice(0, index + 1);
    selectedComponentId.value = null;
  };

  /**
   * 测试数据更新
   */
  const handleTestDataUpdated = (data) => {
    console.log('测试数据已更新:', data);
    // 通知父组件更新测试数据
    emit('test-data-updated', data);
  };

  /**
   * 添加检测到的组件
   */
  const handleAddDetectedComponents = (detectedVars) => {
    if (!detectedVars || detectedVars.length === 0) return;

    const templateMap = new Map();
    COMPONENT_CATEGORIES.forEach((category) => {
      category.components.forEach((comp) => {
        templateMap.set(comp.type, comp);
      });
    });

    detectedVars.forEach((detectedVar) => {
      // 检查是否已存在
      const exists = components.value.some((c) => c.fieldName === detectedVar.name);
      if (exists) return;

      // 智能类型推断
      const smartType = getSuggestedType(detectedVar);
      const template = templateMap.get(smartType);

      if (!template) return;

      // 创建新组件
      const newComponent = {
        id: `comp_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
        fieldName: detectedVar.name,
        type: smartType,
        schema: {
          type: smartType,
          title: detectedVar.name,
          description: `模板变量: 在${detectedVar.files?.length || 0}个文件中使用`,
          required: false,
          default: getDefaultValueForType(smartType),
          insertText: `{{ ${detectedVar.name} }}`,
        },
        template,
        children: smartType === 'object' || smartType === 'object_arr' ? [] : undefined,
      };

      // 为特定类型添加额外字段
      if (smartType === 'object') {
        newComponent.schema.properties = {};
      } else if (smartType === 'object_arr') {
        newComponent.schema.items = {
          type: 'object',
          properties: {},
        };
      }

      components.value.push(newComponent);
    });
  };

  /**
   * 删除未使用的组件
   */
  const handleRemoveUnusedComponents = (variableNames) => {
    if (!variableNames || variableNames.length === 0) return;

    let removedCount = 0;
    variableNames.forEach((name) => {
      const index = components.value.findIndex((c) => c.fieldName === name);
      if (index > -1) {
        components.value.splice(index, 1);
        removedCount++;
      }
    });

    if (removedCount > 0) {
      message.success(`已删除 ${removedCount} 个未使用的变量`);
    }
  };

  /**
   * 根据上下文获取推荐类型
   */
  const getSuggestedType = (detectedVar) => {
    if (!detectedVar.contexts || detectedVar.contexts.length === 0) {
      return detectedVar.type === 'unknown' ? 'string' : detectedVar.type;
    }

    const contexts = detectedVar.contexts;
    const hasRangeContext = contexts.some((c) => c.includes('range'));
    const hasIfContext = contexts.some((c) => c.includes('if'));
    const hasIndexContext = contexts.some((c) => c.includes('['));
    const hasNestedContext = contexts.some((c) => c.includes('.') && !c.match(/\{\{\.[^.]+\}\}$/));

    if (hasRangeContext) {
      return 'array';
    }

    if (hasIndexContext) {
      return 'array';
    }

    if (hasNestedContext) {
      return 'object';
    }

    if (hasIfContext) {
      return 'boolean';
    }

    return detectedVar.type === 'unknown' ? 'string' : detectedVar.type;
  };

  /**
   * 获取类型的默认值
   */
  const getDefaultValueForType = (type) => {
    const defaults = {
      string: '',
      integer: 0,
      number: 0,
      boolean: false,
      array: [],
      object: {},
      object_arr: [],
      enum: '',
      secret: '',
    };
    return defaults[type] !== undefined ? defaults[type] : '';
  };

  /**
   * 关闭 Drawer
   */
  const handleCloseDrawer = () => {
    drawerVisible.value = false;
  };

  /**
   * 保存草稿
   */
  const saveDraft = () => {
    // 防止在同步或保存期间重复保存
    if (isSyncingOrSaving) {
      return;
    }

    const schema = componentsToSchema();
    schemaStore.saveDraft('quick', schema);
  };

  /**
   * 加载模板变量
   * 从服务器或草稿加载变量设计
   */
  const loadVariables = async () => {
    if (!props.templateId) {
      console.log('没有模板ID，跳过加载');
      return;
    }

    // 防止重复加载
    if (isLoading.value) {
      return;
    }

    isLoading.value = true;

    try {
      // 等待 schemaStore 完成加载
      await schemaStore.loadServerSchema();
      schemaStore.loadDrafts();

      // 使用 nextTick 确保响应式数据已更新
      await nextTick();

      // 获取数据
      const quickDraftData = schemaStore.quickDraft.value;
      const serverSchemaData = schemaStore.serverSchema.value;

      // 如果有快速模式草稿，优先加载草稿
      if (Object.keys(quickDraftData).length > 0) {
        componentManager.schemaToComponents(quickDraftData);
      } else if (Object.keys(serverSchemaData).length > 0) {
        componentManager.schemaToComponents(serverSchemaData);
      } else {
        // 都没有，清空组件列表
        resetComponents();
      }

      // 等待组件列表更新后，默认选中第一个组件
      await nextTick();

      if (components.value.length > 0) {
        const firstComponent = components.value[0];
        selectedComponentId.value = firstComponent.id;
      }
    } catch (error) {
      console.error('加载变量失败:', error);
      message.error('加载变量失败: ' + error.message);
    } finally {
      isLoading.value = false;
    }
  };

  // ========== 生命周期 ==========

  onMounted(() => {
    // 组件挂载时，设置 templateId 并加载数据
    if (props.templateId) {
      schemaStore.setTemplateId(props.templateId);
    }
    loadVariables();
  });

  // ========== 监听 drawer 打开状态 ==========

  watch(
    () => props.show,
    (newVal) => {
      // 当 drawer 打开时，重新加载模板变量
      if (newVal) {
        // 禁止 body 滚动
        document.body.style.overflow = 'hidden';

        // 确保使用最新的 templateId
        if (props.templateId) {
          schemaStore.setTemplateId(props.templateId);
        }
        loadVariables();
      } else {
        // 恢复 body 滚动
        document.body.style.overflow = '';
      }
    }
  );

  // ========== 监听 templateId 变化 ==========

  watch(
    () => props.templateId,
    (newId) => {
      // 当 templateId 变化时，更新 store
      if (newId) {
        schemaStore.setTemplateId(newId);
      }
    }
  );

  // ========== 监听变化自动保存草稿 ==========

  let saveDraftTimer = null;
  let lastSaveTime = 0;

  watch(components, () => {
    // 加载期间不保存草稿
    if (isLoading.value) {
      return;
    }

    // 更新期间不保存草稿
    if (isUpdating) {
      return;
    }

    // 同步或保存期间不保存草稿，避免循环触发
    if (isSyncingOrSaving) {
      return;
    }

    // 如果在类型更新后的冷却期（1秒）内，不保存草稿
    const timeSinceLastTypeUpdate = Date.now() - lastTypeUpdateTime;
    if (timeSinceLastTypeUpdate < 1000) {
      return;
    }

    // 节流：500ms内只保存一次
    const now = Date.now();
    if (now - lastSaveTime < 500) {
      if (saveDraftTimer) {
        clearTimeout(saveDraftTimer);
      }
      saveDraftTimer = setTimeout(() => {
        lastSaveTime = Date.now();
        saveDraft();
        saveDraftTimer = null;
      }, 500);
      return;
    }

    // 立即保存
    lastSaveTime = now;
    saveDraft();
  });

  // ========== 暴露方法给父组件 ==========

  defineExpose({
    showTestDataModal: () => {
      handleShowTestData();
    },
    clearDrafts: () => {
      schemaStore.clearDrafts();
    },
  });
</script>

<style scoped>
  /* Drawer 容器布局 */
  .drawer-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Drawer 样式 - 确保独立滚动 */
  :deep(.n-drawer-body) {
    display: flex !important;
    flex-direction: column !important;
    overflow: hidden !important;
  }

  /* 主内容包裹层 - 确保不滚动 */
  .main-content-wrapper {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* 统一所有面板的 header 样式，确保对齐 */
  :deep(.panel-header),
  :deep(.canvas-header),
  :deep(.preview-header) {
    padding: 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fff;
    display: flex;
    align-items: center;
    height: 56px;
    box-sizing: border-box;
    margin: 0;
    flex-shrink: 0;
  }

  /* 统一所有面板的内容区域样式，确保独立滚动 */
  :deep(.component-list),
  :deep(.canvas-area),
  :deep(.tree-content),
  :deep(.property-content),
  :deep(.preview-content),
  :deep(.schema-editor) {
    flex: 1 !important;
    overflow-y: auto !important;
    overflow-x: hidden !important;
    min-height: 0 !important;
  }

  /* 三栏布局 */
  .three-column-layout {
    display: flex !important;
    height: 100% !important;
    gap: 0;
    flex: 1 !important;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  .layout-column {
    border-right: 1px solid #e0e0e0;
    transition: all 0.3s;
    min-height: 0 !important;
    overflow: hidden !important;
    display: flex !important;
    flex-direction: column !important;
  }

  .layout-column:last-child {
    border-right: none;
  }

  /* 设计模式容器 */
  .design-canvas-container {
    display: flex !important;
    flex-direction: row !important;
    height: 100% !important;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  /* 变量树编辑器容器 */
  .tree-editor-container {
    display: flex !important;
    flex-direction: row !important;
    height: 100% !important;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  /* 画布包装器（包含面包屑和画布） */
  .canvas-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  /* 面包屑导航 */
  .breadcrumb-nav {
    padding: 8px 16px;
    background: #fff;
    border-bottom: 1px solid #e0e0e0;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .breadcrumb-nav :deep(.n-breadcrumb) {
    flex: 1;
  }

  .breadcrumb-nav :deep(.n-breadcrumb-item) {
    cursor: pointer;
    font-size: 13px;
  }

  .breadcrumb-nav :deep(.n-breadcrumb-item:hover) {
    color: #18a058;
  }

  /* 表单预览容器 */
  .form-preview-container {
    height: 100% !important;
    display: flex !important;
    flex-direction: column !important;
    background: #fafafa;
    min-height: 0 !important;
    overflow: hidden !important;
  }

  .preview-header {
    padding: 16px;
    border-bottom: 1px solid #e0e0e0;
    background: #fff;
    flex-shrink: 0 !important;
  }

  .preview-content {
    flex: 1 !important;
    overflow-y: auto !important;
    overflow-x: hidden !important;
    padding: 16px;
    background: #f9f9f9;
    min-height: 0 !important;
  }
</style>

<style>
  /* 强制禁止页面级滚动 */
  .n-drawer {
    overflow: hidden !important;
  }

  .n-drawer-container {
    overflow: hidden !important;
  }

  .n-drawer-content-wrapper {
    overflow: hidden !important;
  }

  .n-drawer-body {
    overflow: hidden !important;
    display: flex !important;
    flex-direction: column !important;
  }

  /* 确保 drawer-container 不会超出视口 */
  .drawer-container {
    max-height: 100vh !important;
    overflow: hidden !important;
  }
</style>
