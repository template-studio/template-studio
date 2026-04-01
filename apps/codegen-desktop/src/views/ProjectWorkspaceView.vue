<template>
  <div class="project-workspace-view">
    <a-card title="项目工作区" :bordered="false">
      <a-empty description="项目工作区功能开发中..." />
    </a-card>
  </div>
</template>

<script setup>
import { useRoute } from 'vue-router'

const route = useRoute()
const projectId = route.params.id
</script>

<style scoped>
.project-workspace-view {
  padding: var(--spacing-lg);
}
</style>
