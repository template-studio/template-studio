<template>
  <a-modal
    v-model:open="visible"
    :title="title"
    :ok-text="okText"
    :cancel-text="cancelText"
    :ok-type="okType"
    @ok="handleOk"
    @cancel="handleCancel"
  >
    <p>{{ content }}</p>
  </a-modal>
</template>

<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  open: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: '确认操作'
  },
  content: {
    type: String,
    default: '确定要执行此操作吗？'
  },
  okText: {
    type: String,
    default: '确定'
  },
  cancelText: {
    type: String,
    default: '取消'
  },
  okType: {
    type: String,
    default: 'primary'
  }
})

const emit = defineEmits(['update:open', 'ok', 'cancel'])

const visible = ref(props.open)

// 监听外部值变化
watch(() => props.open, (newVal) => {
  visible.value = newVal
})

// 监听内部值变化
watch(visible, (newVal) => {
  emit('update:open', newVal)
})

// 确定
const handleOk = () => {
  emit('ok')
  visible.value = false
}

// 取消
const handleCancel = () => {
  emit('cancel')
  visible.value = false
}
</script>

<style scoped>
/* 样式已由 ant-design-vue 的 a-modal 提供 */
</style>
