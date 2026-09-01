<template>
  <div class="condition-builder" v-if="localValue">
    <!-- 根条件类型选择 -->
    <div class="condition-type-selector">
      <a-select
        v-model:value="localValue.type"
        :options="conditionTypeOptions"
        @change="handleTypeChange"
        placeholder="请选择条件类型"
        :get-popup-container="getPopupContainer"
      />
    </div>

    <!-- IF 条件 -->
    <template v-if="localValue.type === 'if'">
      <div class="condition-row">
        <span class="condition-label">变量</span>
        <a-select
          v-model:value="localValue.variable"
          :options="variables"
          placeholder="选择变量"
          show-search
          :disabled="variables.length === 0"
        />
        <span v-if="variables.length === 0" class="empty-hint">暂无可选变量</span>
      </div>

      <div class="condition-row">
        <span class="condition-label">操作符</span>
        <a-select
          v-model:value="localValue.operator"
          :options="operatorOptions"
          placeholder="选择操作符"
        />
      </div>

      <div class="condition-row">
        <span class="condition-label">值</span>
        <a-input v-model:value="localValue.value" placeholder="输入比较值" allow-clear />
      </div>

      <!-- 快捷条件按钮 -->
      <div v-if="localValue.variable" class="quick-conditions">
        <div class="quick-conditions-label">快捷设置：</div>

        <!-- 布尔类型变量 -->
        <template v-if="selectedVariableType === 'boolean' || selectedVariableType === '布尔值'">
          <a-space :size="8">
            <a-button size="small" type="primary" @click="setQuickCondition('eq', true)">
              等于 true
            </a-button>
            <a-button size="small" @click="setQuickCondition('eq', false)"> 等于 false </a-button>
            <a-button size="small" @click="setQuickCondition('ne', true)"> 不等于 true </a-button>
            <a-button size="small" @click="setQuickCondition('ne', false)"> 不等于 false </a-button>
          </a-space>
        </template>

        <!-- 数字类型变量 -->
        <template
          v-else-if="
            selectedVariableType === 'number' ||
            selectedVariableType === 'integer' ||
            selectedVariableType === '数字' ||
            selectedVariableType === '整数'
          "
        >
          <a-space :size="8">
            <a-button size="small" type="primary" @click="setQuickCondition('eq', 0)">
              等于 0
            </a-button>
            <a-button size="small" @click="setQuickCondition('gt', 0)"> 大于 0 </a-button>
            <a-button size="small" @click="setQuickCondition('gte', 1)"> 大于等于 1 </a-button>
            <a-button size="small" @click="setQuickCondition('ne', 0)"> 不等于 0 </a-button>
          </a-space>
        </template>

        <!-- 字符串类型变量 -->
        <template v-else>
          <a-space :size="8">
            <a-button size="small" @click="setQuickCondition('ne', '')"> 不为空 </a-button>
            <a-button size="small" @click="setQuickCondition('eq', 'true')"> 等于 "true" </a-button>
            <a-button size="small" @click="setQuickCondition('eq', 'false')">
              等于 "false"
            </a-button>
          </a-space>
        </template>
      </div>
    </template>

    <!-- AND/OR 条件 -->
    <template v-else-if="localValue.type === 'and' || localValue.type === 'or'">
      <div class="condition-group">
        <div
          v-for="(condition, index) in localValue.conditions"
          :key="index"
          class="nested-condition"
        >
          <div class="condition-header">
            <span class="condition-index">条件 {{ index + 1 }}</span>
            <a-button size="small" danger @click="removeCondition(index)">
              <template #icon><close-icon /></template>
            </a-button>
          </div>
          <condition-builder v-model="localValue.conditions[index]" :variables="variables" />
        </div>
        <a-button size="small" block @click="addCondition" style="border-style: dashed">
          <template #icon><plus-icon /></template>
          添加条件
        </a-button>
      </div>
    </template>

    <!-- NOT 条件 -->
    <template v-else-if="localValue.type === 'not'">
      <div class="condition-group">
        <condition-builder v-model="localValue.condition" :variables="variables" />
      </div>
    </template>

    <!-- SWITCH 条件 -->
    <template v-else-if="localValue.type === 'switch'">
      <div class="condition-row">
        <span class="condition-label">变量</span>
        <a-select
          v-model:value="localValue.variable"
          :options="variables"
          placeholder="选择变量"
          show-search
        />
      </div>

      <div class="condition-group">
        <div v-for="(caseItem, index) in localValue.cases" :key="index" class="switch-case">
          <div class="case-header">
            <span>分支 {{ index + 1 }}</span>
            <a-button size="small" danger @click="removeCase(index)">
              <template #icon><close-icon /></template>
            </a-button>
          </div>
          <div class="case-value">
            <a-input v-model:value="caseItem.value" placeholder="匹配值" />
          </div>
        </div>
        <a-button size="small" block @click="addCase" style="border-style: dashed">
          <template #icon><plus-icon /></template>
          添加分支
        </a-button>
      </div>
    </template>
  </div>
</template>

<script setup>
  import { ref, watch, computed } from 'vue';
  import { Add as PlusIcon, Close as CloseIcon } from '@/icons/ionicons5';
  import { ConditionTypes, Operators, ConditionTypeLabels, OperatorLabels } from '@/api/editor/conditions';

  const props = defineProps({
    modelValue: {
      type: [Object, null],
      default: null,
    },
    variables: {
      type: Array,
      default: () => [],
    },
  });

  const emit = defineEmits(['update:modelValue']);

  // 使用 ref 而不是 computed，以支持嵌套属性的修改
  const localValue = ref(null);

  // 只监听 props.modelValue，单向同步到 localValue
  watch(
    () => props.modelValue,
    (newValue) => {
      if (newValue) {
        localValue.value = JSON.parse(JSON.stringify(newValue));
      } else {
        localValue.value = null;
      }
    },
    { deep: true, immediate: true }
  );

  // 监听 localValue 的变化，同步到父组件（但避免循环）
  // 只在用户通过 UI 修改时触发，不监听从 props 来的更新
  watch(
    localValue,
    (newValue) => {
      // 只在有值时才同步，避免 null 导致的循环
      if (newValue && JSON.stringify(newValue) !== JSON.stringify(props.modelValue)) {
        emit('update:modelValue', newValue);
      }
    },
    { deep: true }
  );

  // 确保下拉菜单渲染到 body，避免被裁剪
  const getPopupContainer = () => document.body;

  const conditionTypeOptions = Object.values(ConditionTypes).map((type) => ({
    label: ConditionTypeLabels[type],
    value: type,
  }));

  const operatorOptions = Object.values(Operators).map((op) => ({
    label: OperatorLabels[op],
    value: op,
  }));

  const handleTypeChange = (type) => {
    // 根据类型初始化默认结构
    switch (type) {
      case 'if':
        localValue.value = {
          type: 'if',
          variable: '',
          operator: 'eq',
          value: '',
        };
        break;
      case 'and':
      case 'or':
        localValue.value = {
          type,
          conditions: [{ type: 'if', variable: '', operator: 'eq', value: '' }],
        };
        break;
      case 'not':
        localValue.value = {
          type: 'not',
          condition: { type: 'if', variable: '', operator: 'eq', value: '' },
        };
        break;
      case 'switch':
        localValue.value = {
          type: 'switch',
          variable: '',
          cases: [{ value: '' }],
        };
        break;
    }
  };

  const addCondition = () => {
    if (!localValue.value.conditions) {
      localValue.value.conditions = [];
    }
    localValue.value.conditions.push({
      type: 'if',
      variable: '',
      operator: 'eq',
      value: '',
    });
  };

  const removeCondition = (index) => {
    localValue.value.conditions.splice(index, 1);
  };

  const addCase = () => {
    if (!localValue.value.cases) {
      localValue.value.cases = [];
    }
    localValue.value.cases.push({ value: '' });
  };

  const removeCase = (index) => {
    localValue.value.cases.splice(index, 1);
  };

  // 获取当前选中变量的类型
  const selectedVariableType = computed(() => {
    if (!localValue.value || !localValue.value.variable) {
      return null;
    }
    const variable = props.variables.find((v) => v.value === localValue.value.variable);
    return variable?.type || null;
  });

  // 设置快捷条件
  const setQuickCondition = (operator, value) => {
    if (!localValue.value) return;

    localValue.value.operator = operator;
    localValue.value.value = value;
  };
</script>

<style scoped>
  .condition-builder {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .condition-type-selector {
    margin-bottom: 8px;
  }

  .condition-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .condition-label {
    min-width: 60px;
    font-size: 13px;
    color: var(--editor-muted, #666);
  }

  .empty-hint {
    font-size: 12px;
    color: #ff9900;
    margin-left: 8px;
  }

  .condition-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-left: 16px;
    border-left: 2px solid var(--editor-border, #e0e0e0);
  }

  .nested-condition {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .condition-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .condition-index {
    font-size: 12px;
    font-weight: 600;
    color: var(--editor-muted, #666);
  }

  .switch-case {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .case-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    color: var(--editor-muted, #666);
  }

  .case-value {
    flex: 1;
  }

  .quick-conditions {
    margin-top: 8px;
    padding: 12px;
    background: #f5f7fa;
    border-radius: 6px;
    border: 1px solid #e0e4e9;
  }

  .quick-conditions-label {
    font-size: 12px;
    color: var(--editor-muted, #666);
    margin-bottom: 8px;
    font-weight: 500;
  }

  .quick-conditions :deep(.ant-btn) {
    font-size: 12px;
    padding: 4px 12px;
    height: auto;
  }
</style>
