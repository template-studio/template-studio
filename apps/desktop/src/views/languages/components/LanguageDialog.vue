<template>
  <a-modal
    v-model:open="dialogVisible"
    :title="mode === 'edit' ? '编辑语言' : '添加语言'"
    width="500px"
    :confirm-loading="submitting"
    @ok="handleSubmit"
  >
    <a-form
      ref="formRef"
      :model="formData"
      :rules="formRules"
      layout="vertical"
      @finish="handleSubmit"
    >
      <a-form-item label="语言名称" name="name">
        <a-input
          v-model:value="formData.name"
          placeholder="例如：Rust, Go, TypeScript"
          size="large"
        />
      </a-form-item>

      <a-form-item label="图标（Emoji）" name="icon">
        <a-input
          v-model:value="formData.icon"
          placeholder="输入 emoji 图标，例如：🦀, 🐹, 💛"
          size="large"
        />
        <div class="emoji-hint" v-if="emojiSuggestions.length > 0">
          <span
            v-for="emoji in emojiSuggestions"
            :key="emoji"
            @click="formData.icon = emoji"
            class="emoji-suggestion"
          >
            {{ emoji }}
          </span>
        </div>
      </a-form-item>

      <a-form-item label="颜色" name="color">
        <a-select
          v-model:value="formData.color"
          placeholder="选择颜色"
          size="large"
        >
          <a-select-option value="default">
            <span class="color-option">
              <span class="color-box" style="background: #d9d9d9;"></span>
              默认灰色
            </span>
          </a-select-option>
          <a-select-option value="red">
            <span class="color-option">
              <span class="color-box" style="background: #f5222d;"></span>
              红色
            </span>
          </a-select-option>
          <a-select-option value="orange">
            <span class="color-option">
              <span class="color-box" style="background: #fa8c16;"></span>
              橙色
            </span>
          </a-select-option>
          <a-select-option value="gold">
            <span class="color-option">
              <span class="color-box" style="background: #faad14;"></span>
              金色
            </span>
          </a-select-option>
          <a-select-option value="green">
            <span class="color-option">
              <span class="color-box" style="background: #52c41a;"></span>
              绿色
            </span>
          </a-select-option>
          <a-select-option value="cyan">
            <span class="color-option">
              <span class="color-box" style="background: #13c2c2;"></span>
            青色
            </span>
          </a-select-option>
          <a-select-option value="blue">
            <span class="color-option">
              <span class="color-box" style="background: #3e7bfa;"></span>
              蓝色
            </span>
          </a-select-option>
          <a-select-option value="purple">
            <span class="color-option">
              <span class="color-box" style="background: #722ed1;"></span>
              紫色
            </span>
          </a-select-option>
          <a-select-option value="pink">
            <span class="color-option">
              <span class="color-box" style="background: #eb2f96;"></span>
              粉色
            </span>
          </a-select-option>
        </a-select>
      </a-form-item>

      <a-form-item label="描述" name="description">
        <a-textarea
          v-model:value="formData.description"
          placeholder="简要描述该语言的用途（可选）"
          size="large"
          :rows="3"
        />
      </a-form-item>
    </a-form>

    <!-- 对话框底部按钮 -->
    <template #footer>
      <a-button @click="dialogVisible = false">取消</a-button>
      <a-button type="primary" :loading="submitting" @click="handleSubmit">
        {{ mode === 'edit' ? '保存' : '添加' }}
      </a-button>
    </template>
  </a-modal>
</template>

<script setup>
import { ref, reactive, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import * as languagesApi from '@/api/languages'

const props = defineProps({
  open: { type: Boolean, default: false },
  mode: { type: String, default: 'create' },
  language: { type: Object, default: null }
})

const emit = defineEmits(['update:open', 'saved'])

// 对话框可见性
const dialogVisible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

const submitting = ref(false)
const formRef = ref()

// 表单数据
const formData = reactive({
  name: '',
  icon: '',
  color: 'blue',
  description: ''
})

// 表单验证规则
const formRules = {
  name: [
    { required: true, message: '请输入语言名称', trigger: 'blur' },
    { min: 2, max: 20, message: '语言名称长度应在 2-20 个字符', trigger: 'blur' }
  ],
  icon: [
    { required: true, message: '请选择图标', trigger: 'change' }
  ]
}

// Emoji 建议
const emojiSuggestions = computed(() => {
  const name = formData.name.toLowerCase()
  const suggestions = []

  if (name.includes('rust') || name.includes('系统')) suggestions.push('🦀')
  if (name.includes('go') || name.includes('golang')) suggestions.push('🐹')
  if (name.includes('python')) suggestions.push('🐍')
  if (name.includes('java')) suggestions.push('☕')
  if (name.includes('javascript') || name.includes('js')) suggestions.push('💛')
  if (name.includes('typescript') || name.includes('ts')) suggestions.push('💠')
  if (name.includes('c++')) suggestions.push('⚡')
  if (name.includes('c#')) suggestions.push('🔷')
  if (name.includes('swift')) suggestions.push('🍎')
  if (name.includes('kotlin')) suggestions.push('🤖')
  if (name.includes('dart')) suggestions.push('🎯')
  if (name.includes('php')) suggestions.push('🐘')
  if (name.includes('ruby')) suggestions.push('💎')

  return suggestions
})

// 监听打开事件，初始化表单
watch(() => props.open, (val) => {
  if (val) {
    if (props.mode === 'edit' && props.language) {
      Object.assign(formData, {
        name: props.language.name,
        icon: props.language.icon || '',
        color: props.language.color || 'blue',
        description: props.language.description || ''
      })
    } else {
      Object.assign(formData, {
        name: '',
        icon: '',
        color: 'blue',
        description: ''
      })
    }
  }
})

// 提交表单
const handleSubmit = async () => {
  try {
    await formRef.value.validate()
  } catch (error) {
    return
  }

  submitting.value = true
  try {
    const data = {
      name: formData.name,
      icon: formData.icon,
      color: formData.color,
      description: formData.description
    }

    if (props.mode === 'edit') {
      await languagesApi.updateLanguage(props.language.id, data)
      message.success('语言更新成功')
    } else {
      await languagesApi.createLanguage(data)
      message.success('语言添加成功')
    }
    dialogVisible.value = false
    emit('saved')
  } catch (error) {
    message.error('操作失败: ' + error)
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.emoji-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.emoji-suggestion {
  display: inline-block;
  font-size: 24px;
  margin: 0 4px;
  cursor: pointer;
  transition: transform 0.2s;
  padding: 4px;
  border-radius: 4px;
}

.emoji-suggestion:hover {
  transform: scale(1.2);
  background: var(--color-hover);
}

.color-option {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-box {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: 1px solid var(--color-border);
}
</style>
