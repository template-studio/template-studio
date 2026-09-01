<template>
  <div class="preset-variable-manager" :class="{ 'compact-mode': compact }">
    <!-- 头部操作栏 -->
    <div class="manager-header">
      <div class="header-left">
        <h3 v-if="!compact">预设变量订阅</h3>
        <h4 v-else>预设变量</h4>
        <p v-if="!compact" class="description">从预设变量库中订阅变量到当前模板</p>
      </div>
      <div class="header-right">
        <a-button
          :type="compact ? 'default' : 'primary'"
          :size="compact ? 'small' : 'middle'"
          @click="showSubscribeModal = true"
          :loading="loading"
        >
          <template #icon><AddOutline /></template>
          {{ compact ? '订阅' : '订阅预设变量' }}
        </a-button>
      </div>
    </div>

    <!-- 已订阅的预设变量列表 -->
    <a-spin :spinning="loading">
      <div class="subscribed-list">
        <a-empty v-if="subscribedList.length === 0" description="暂无订阅的预设变量">
          <a-button size="small" @click="showSubscribeModal = true">立即订阅</a-button>
        </a-empty>

        <div v-else>
          <div class="preset-group" v-for="preset in subscribedList" :key="preset.id">
            <!-- 预设变量包头部 -->
            <div class="preset-header">
              <div class="preset-title">
                <span class="preset-name">{{ preset.presetName }}</span>
                <span class="variable-count">({{ preset.variables?.length || 0 }} 个变量)</span>
              </div>
              <div class="preset-actions">
                <a-popconfirm
                  title="确定要取消订阅这个预设变量包吗？"
                  @confirm="unsubscribe(preset)"
                >
                  <a-button size="small" danger :loading="preset.unsubscribing">
                    取消订阅
                  </a-button>
                </a-popconfirm>
              </div>
            </div>

            <!-- 预设变量包中的具体变量 -->
            <div class="variables-list" v-if="preset.variables && preset.variables.length > 0">
              <div class="variable-item" v-for="variable in preset.variables" :key="variable.path">
                <div class="variable-header">
                  <div class="variable-info">
                    <span class="variable-path">{{ variable.path }}</span>
                    <span class="variable-type">{{ variable.type }}</span>
                  </div>
                  <div class="variable-copy">
                    <a-button size="small" @click="copyVariablePath(variable.path)">
                      复制
                    </a-button>
                  </div>
                </div>
                <div class="variable-content">
                  <div class="variable-display-name" v-if="variable.displayName">
                    {{ variable.displayName }}
                  </div>
                  <div class="variable-description" v-if="variable.description">
                    {{ variable.description }}
                  </div>
                  <div class="variable-default" v-if="variable.default">
                    <span class="label">默认值:</span>
                    <span class="value">{{ variable.default }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </a-spin>

    <!-- 订阅预设变量弹窗 -->
    <a-modal
      v-model:open="showSubscribeModal"
      title="订阅预设变量"
      :width="800"
      :mask-closable="false"
      :footer="null"
    >
      <!-- 搜索栏 -->
      <div class="search-bar">
        <a-input
          v-model:value="searchKeyword"
          placeholder="搜索预设变量..."
          allow-clear
          @change="searchPresets"
        >
          <template #prefix><SearchOutline /></template>
        </a-input>
      </div>

      <!-- 可用预设变量列表 -->
      <a-spin :spinning="presetsLoading">
        <div class="available-presets">
          <a-empty v-if="availablePresets.length === 0" description="暂无可用的预设变量" />

          <div v-else class="presets-list">
            <div class="preset-item" v-for="preset in availablePresets" :key="preset.id">
              <div class="preset-content">
                <a-checkbox
                  :checked="isPresetSelected(preset.id)"
                  @change="(e) => togglePreset(preset, e.target.checked)"
                >
                  <div class="preset-info">
                    <div class="preset-name">{{ preset.name }}</div>
                    <div class="preset-description" v-if="preset.description">
                      {{ preset.description }}
                    </div>
                  </div>
                </a-checkbox>
              </div>
            </div>
          </div>
        </div>
      </a-spin>

      <!-- 分页 -->
      <div class="pagination" v-if="totalPresets > pageSize">
        <a-pagination
          v-model:current="currentPage"
          :page-size="pageSize"
          :total="totalPresets"
          :page-size-options="['10', '20', '30']"
          show-size-changer
          @change="(page) => loadAvailablePresets(page)"
          @showSizeChange="(current, size) => handlePageSizeChange(size)"
        />
      </div>

      <template #footer>
        <div class="modal-footer">
          <a-button @click="showSubscribeModal = false">取消</a-button>
          <a-button
            type="primary"
            @click="confirmSubscribe"
            :loading="subscribing"
            :disabled="selectedPresets.length === 0"
          >
            订阅选中的预设 ({{ selectedPresets.length }})
          </a-button>
        </div>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
  import { ref, onMounted, watch } from 'vue';
  import { message } from 'ant-design-vue';
  import { AddOutline, CloseOutline, SearchOutline } from '@/icons/ionicons5';
  import {
    subscribePreset,
    getSubscribedPresets,
    unsubscribePreset,
    getAvailablePresets,
  } from '@/api/editor/templateVariablePresets';

  const props = defineProps({
    templateId: {
      type: [Number, String],
      required: true,
    },
    compact: {
      type: Boolean,
      default: false,
    },
  });

  // 数据状态
  const loading = ref(false);
  const subscribedList = ref([]);
  const showSubscribeModal = ref(false);

  // 可用预设变量相关
  const presetsLoading = ref(false);
  const availablePresets = ref([]);
  const totalPresets = ref(0);
  const currentPage = ref(1);
  const pageSize = ref(20);
  const searchKeyword = ref('');

  // 选中的预设变量
  const selectedPresets = ref([]);

  // 订阅状态
  const subscribing = ref(false);

  // 加载已订阅的预设变量
  const loadSubscribedPresets = async () => {
    loading.value = true;
    try {
      const response = await getSubscribedPresets(props.templateId);
      subscribedList.value = response.data.data.list || [];
    } catch (error) {
      console.error('加载订阅列表失败:', error);
      message.error('加载订阅列表失败');
    } finally {
      loading.value = false;
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
      currentPage.value = data.pageNum || 1;
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

  // 变量选择相关
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

  // 取消订阅
  const unsubscribe = async (item) => {
    item.unsubscribing = true;
    try {
      await unsubscribePreset(props.templateId, item.presetId);
      message.success('取消订阅成功');
      await loadSubscribedPresets();
    } catch (error) {
      console.error('取消订阅失败:', error);
      message.error('取消订阅失败');
    } finally {
      item.unsubscribing = false;
    }
  };

  // 复制变量路径
  const copyVariablePath = async (path) => {
    try {
      await navigator.clipboard.writeText(`{{${path}}}`);
      message.success('变量路径已复制到剪贴板');
    } catch (error) {
      console.error('复制失败:', error);
      message.error('复制失败');
    }
  };

  // 初始化
  onMounted(() => {
    loadSubscribedPresets();
  });

  // 监听弹窗打开，加载可用预设变量
  watch(() => showSubscribeModal.value, () => {
    if (showSubscribeModal.value) {
      selectedPresets.value = [];
      currentPage.value = 1;
      searchKeyword.value = '';
      loadAvailablePresets();
    }
  });
</script>

<style scoped>
  .preset-variable-manager {
    padding: 16px;
  }

  .manager-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 24px;
  }

  .header-left h3 {
    margin: 0 0 4px 0;
    color: #333;
  }

  .header-left .description {
    margin: 0;
    color: #666;
    font-size: 14px;
  }

  .subscribed-list .list-item {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 12px;
    background: var(--editor-panel-bg, #fff);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .item-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preset-name {
    font-weight: 500;
    color: #333;
  }

  .item-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .item-content {
    padding-left: 0;
  }

  .mapping-info,
  .info-row {
    display: flex;
    margin-bottom: 4px;
  }

  .mapping-info .label,
  .info-row .label {
    width: 60px;
    color: #666;
    font-size: 12px;
  }

  .mapping-info .value,
  .info-row .value {
    color: #333;
    font-size: 12px;
  }

  .search-bar {
    margin-bottom: 16px;
  }

  .available-presets {
    max-height: 400px;
    overflow-y: auto;
  }

  .preset-item {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 16px;
  }

  .preset-header h4 {
    margin: 0 0 8px 0;
    color: #333;
  }

  .preset-description {
    margin: 0 0 16px 0;
    color: #666;
    font-size: 14px;
  }

  .variable-item {
    padding: 8px 0;
    border-bottom: 1px solid #f0f0f0;
  }

  .variable-item:last-child {
    border-bottom: none;
  }

  .variable-info {
    margin-left: 24px;
  }

  .variable-name {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }

  .path {
    font-family: monospace;
    color: #333;
    background: #f5f5f5;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }

  .display-name {
    color: #666;
    font-size: 12px;
  }

  .variable-desc {
    color: #999;
    font-size: 12px;
    line-height: 1.4;
  }

  .mapping-config {
    margin: 8px 0 0 24px;
    max-width: 200px;
  }

  .pagination {
    margin-top: 16px;
    text-align: center;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  /* 紧凑模式样式 */
  .compact-mode .manager-header {
    margin-bottom: 12px;
  }

  .compact-mode .manager-header h4 {
    font-size: 14px;
    margin-bottom: 2px;
  }

  .compact-mode .subscribed-list .list-item {
    padding: 8px;
    margin-bottom: 8px;
    border-radius: 6px;
  }

  .compact-mode .item-header {
    margin-bottom: 6px;
  }

  .compact-mode .preset-name {
    font-size: 13px;
  }

  .compact-mode .item-actions :deep(.ant-btn) {
    padding: 2px 6px;
    font-size: 11px;
    height: 24px;
  }

  .compact-mode .item-actions :deep(.ant-switch) {
    transform: scale(0.85);
  }

  .compact-mode .mapping-info .label,
  .compact-mode .info-row .label {
    width: 50px;
    font-size: 11px;
  }

  .compact-mode .mapping-info .value,
  .compact-mode .info-row .value {
    font-size: 11px;
  }

  .compact-mode :deep(.ant-tag) {
    font-size: 10px;
    padding: 2px 6px;
  }

  .compact-mode .header-right :deep(.ant-btn) {
    padding: 4px 8px;
    font-size: 12px;
    height: 28px;
  }

  /* 预设变量组样式 */
  .preset-group {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    margin-bottom: 16px;
    background: var(--editor-panel-bg, #fff);
  }

  .preset-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid #f0f0f0;
    background: #fafafa;
    border-radius: 8px 8px 0 0;
  }

  .preset-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preset-name {
    font-weight: 500;
    color: #333;
  }

  .variable-count {
    color: #666;
    font-size: 12px;
  }

  .variables-list {
    padding: 8px;
  }

  .variable-item {
    padding: 8px 12px;
    border: 1px solid #f0f0f0;
    border-radius: 6px;
    margin-bottom: 8px;
    background: #fafafa;
  }

  .variable-item:last-child {
    margin-bottom: 0;
  }

  .variable-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  .variable-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .variable-path {
    font-family: monospace;
    color: #3e7bfa;
    background: #f0f8ff;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 500;
  }

  .variable-type {
    color: #666;
    font-size: 11px;
    background: #f5f5f5;
    padding: 1px 4px;
    border-radius: 3px;
  }

  .variable-content {
    margin-left: 4px;
  }

  .variable-display-name {
    color: #333;
    font-size: 13px;
    font-weight: 500;
    margin-bottom: 2px;
  }

  .variable-description {
    color: #666;
    font-size: 12px;
    line-height: 1.4;
    margin-bottom: 4px;
  }

  .variable-default {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
  }

  .variable-default .label {
    color: #999;
  }

  .variable-default .value {
    color: #666;
    font-family: monospace;
    background: #f5f5f5;
    padding: 1px 4px;
    border-radius: 3px;
  }
</style>
