<template>
  <a-modal
    :open="open"
    title="切换语言"
    ok-text="确认切换"
    cancel-text="取消"
    ok-type="danger"
    @update:open="emit('update:open', $event)"
    @ok="handleConfirm"
  >
    <a-alert
      message="警告：切换语言将导致当前映射丢失"
      description="切换语言后，项目当前的自定义映射将被删除，并从系统默认映射重新复制。此操作不可恢复。"
      type="warning"
      show-icon
      class="switch-language-alert"
    />
    <a-form layout="vertical">
      <a-form-item label="前端语言">
        <a-select v-model:value="switchForm.frontendLanguageId" placeholder="选择前端语言" allow-clear>
          <a-select-option v-for="lang in languages" :key="lang.id" :value="lang.id">
            {{ lang.icon }} {{ lang.name }}
          </a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="后端语言">
        <a-select v-model:value="switchForm.backendLanguageId" placeholder="选择后端语言" allow-clear>
          <a-select-option v-for="lang in languages" :key="lang.id" :value="lang.id">
            {{ lang.icon }} {{ lang.name }}
          </a-select-option>
        </a-select>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup>
import { reactive, watch } from 'vue'

const props = defineProps({
  open: {
    type: Boolean,
    default: false
  },
  languages: {
    type: Array,
    default: () => []
  },
  frontendLanguageId: {
    type: [Number, null],
    default: null
  },
  backendLanguageId: {
    type: [Number, null],
    default: null
  }
})

const emit = defineEmits(['update:open', 'switched'])

const switchForm = reactive({
  frontendLanguageId: null,
  backendLanguageId: null
})

// Sync form state from props when dialog opens
watch(
  () => props.open,
  (val) => {
    if (val) {
      switchForm.frontendLanguageId = props.frontendLanguageId
      switchForm.backendLanguageId = props.backendLanguageId
    }
  }
)

const handleConfirm = () => {
  emit('switched', switchForm.frontendLanguageId, switchForm.backendLanguageId)
}
</script>

<style scoped>
.switch-language-alert {
  margin-bottom: 16px;
}
</style>
