<template>
  <div class="drawer-footer">
    <div class="footer-content">
      <div class="footer-stats">
        <span style="color: var(--editor-muted, #999)"> 已添加 {{ componentCount }} 个组件 </span>
        <span v-if="hasUnsavedChanges" style="margin-left: 12px; color: #faad14">
          ⚠️ 有未保存的更改
        </span>
      </div>
    </div>
  </div>
</template>

<script setup>
  // ant-design-vue migration: NText replaced with native span/a-typography-text

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
    border-top: 1px solid var(--editor-border, #e0e0e0);
    background: var(--editor-inset-bg, #fafafa);
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
