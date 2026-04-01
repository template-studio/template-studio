<template>
  <div class="drawer-footer">
    <div class="footer-content">
      <div class="footer-stats">
        <n-text depth="3"> 已添加 {{ componentCount }} 个组件 </n-text>
        <n-text v-if="hasUnsavedChanges" type="warning" style="margin-left: 12px">
          ⚠️ 有未保存的更改
        </n-text>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { NText } from 'naive-ui';

  /**
   * StudioFooter 组件
   * 负责显示统计信息
   */

  // Props
  const props = defineProps({
    componentCount: {
      type: Number,
      default: 0,
    },
    hasUnsavedChanges: {
      type: Boolean,
      default: false,
    },
  });
</script>

<style scoped>
  .drawer-footer {
    width: 100%;
    border-top: 1px solid #e0e0e0;
    background: #fafafa;
    flex-shrink: 0;
  }

  .footer-content {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    padding: 10px 16px;
  }

  .footer-stats {
    display: flex;
    align-items: center;
  }
</style>
