<template>
  <div class="preferences-view">
    <TablePreferencesManager
      v-if="projectId"
      :project-id="projectId"
      @save="onPreferencesSaved"
    />
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import TablePreferencesManager from '@/components/project/TablePreferencesManager.vue'
import { message } from 'ant-design-vue'

const route = useRoute()

// 获取项目 ID
const projectId = computed(() => {
  const id = route.params.id
  return id ? parseInt(id) : null
})

// 规范配置保存后的回调
const onPreferencesSaved = () => {
  message.success('项目规范配置已保存')
}
</script>

<style scoped>
.preferences-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
