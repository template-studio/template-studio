<template>
  <div class="icon-selector">
    <div class="icon-selector-header">
      <div class="header-top">
        <n-input v-model:value="searchText" placeholder="搜索图标..." clearable size="small">
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
      </div>
      <div class="header-info">
        <span class="icon-count">共 {{ allIcons.length }} 个图标</span>
        <span v-if="searchText" class="filtered-count">找到 {{ filteredIcons.length }} 个匹配</span>
      </div>
    </div>

    <div class="icon-list">
      <div
        v-for="icon in filteredIcons"
        :key="icon.name"
        class="icon-item"
        :class="{ selected: selectedIcon === icon.name }"
        @click="selectIcon(icon)"
        :title="icon.name"
      >
        <div class="icon-wrapper">
          <component :is="icon.component" />
        </div>
        <div class="icon-name">{{ formatIconName(icon.name) }}</div>
      </div>
    </div>

    <div v-if="filteredIcons.length === 0" class="no-results">
      <n-empty description="未找到匹配的图标" />
    </div>
  </div>
</template>

<script setup>
  import { ref, computed, onMounted, markRaw } from 'vue';
  import { SearchOutline } from '@vicons/ionicons5';
  import * as IonIcons from '@vicons/ionicons5';

  // 响应式数据
  const searchText = ref('');
  const selectedIcon = ref('');
  const allIcons = ref([]);

  // 计算过滤后的图标
  const filteredIcons = computed(() => {
    if (!searchText.value) {
      return allIcons.value;
    }

    const searchLower = searchText.value.toLowerCase();
    return allIcons.value.filter(
      (icon) =>
        icon.name.toLowerCase().includes(searchLower) ||
        formatIconName(icon.name).toLowerCase().includes(searchLower)
    );
  });

  // 格式化图标名称显示
  const formatIconName = (name) => {
    return name
      .replace(/Outline$/, '')
      .replace(/([A-Z])/g, ' $1')
      .trim();
  };

  // 选择图标
  const selectIcon = (icon) => {
    selectedIcon.value = icon.name;
    emit('select', icon.name);
  };

  // 初始化图标列表 - 动态遍历所有图标
  const initIcons = () => {
    const icons = [];

    // 遍历IonIcons对象的所有属性
    Object.keys(IonIcons).forEach((iconName) => {
      const iconComponent = IonIcons[iconName];

      // 检查是否是有效的图标组件
      if (typeof iconComponent === 'object' && iconName.endsWith('Outline')) {
        icons.push({
          name: iconName,
          component: markRaw(iconComponent),
        });
      }
    });

    // 按名称排序
    icons.sort((a, b) => a.name.localeCompare(b.name));

    allIcons.value = icons;
  };

  // 暴露方法
  const reset = () => {
    selectedIcon.value = '';
    searchText.value = '';
  };

  const setSelectedIcon = (iconName) => {
    selectedIcon.value = iconName;
  };

  // 定义事件
  const emit = defineEmits(['select']);

  // 暴露方法给父组件
  defineExpose({
    reset,
    setSelectedIcon,
  });

  // 组件挂载时初始化
  onMounted(() => {
    initIcons();
  });
</script>

<style scoped>
  .icon-selector {
    width: 100%;
    max-height: 600px;
    display: flex;
    flex-direction: column;
    background: #fff;
    border-radius: 8px;
  }

  .icon-selector-header {
    padding: 20px;
    border-bottom: 1px solid #f0f0f0;
    background: #fafafa;
    border-radius: 8px 8px 0 0;
  }

  .header-top {
    margin-bottom: 8px;
  }

  .header-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    color: #666;
  }

  .icon-count {
    font-weight: 500;
  }

  .filtered-count {
    color: #18a058;
  }

  .icon-list {
    flex: 1;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
    gap: 12px;
    padding: 20px;
    max-height: 500px;
    background: #fff;
  }

  .icon-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px 12px;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.3s ease;
    border: 2px solid transparent;
    background: #fafafa;
  }

  .icon-item:hover {
    background-color: #f0f9ff;
    border-color: #18a058;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(24, 160, 88, 0.15);
  }

  .icon-item.selected {
    background-color: #e8f5e8;
    border-color: #18a058;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(24, 160, 88, 0.2);
  }

  .icon-item:active {
    transform: translateY(0);
  }

  .icon-wrapper {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 10px;
  }

  .icon-wrapper :deep(svg) {
    width: 28px;
    height: 28px;
    color: #666;
    transition: all 0.3s ease;
  }

  .icon-item:hover .icon-wrapper :deep(svg) {
    color: #18a058;
    transform: scale(1.1);
  }

  .icon-item.selected .icon-wrapper :deep(svg) {
    color: #18a058;
    transform: scale(1.1);
  }

  .icon-name {
    font-size: 12px;
    color: #666;
    text-align: center;
    line-height: 1.3;
    word-break: break-word;
    font-weight: 500;
    transition: all 0.3s ease;
  }

  .icon-item:hover .icon-name {
    color: #18a058;
  }

  .icon-item.selected .icon-name {
    color: #18a058;
    font-weight: 600;
  }

  .icon-item.selected .icon-name {
    color: #18a058;
    font-weight: 500;
  }

  .no-results {
    padding: 40px 20px;
    text-align: center;
  }

  /* 滚动条样式 */
  .icon-list::-webkit-scrollbar {
    width: 6px;
  }

  .icon-list::-webkit-scrollbar-track {
    background: #f1f1f1;
    border-radius: 3px;
  }

  .icon-list::-webkit-scrollbar-thumb {
    background: #c1c1c1;
    border-radius: 3px;
  }

  .icon-list::-webkit-scrollbar-thumb:hover {
    background: #a8a8a8;
  }
</style>
