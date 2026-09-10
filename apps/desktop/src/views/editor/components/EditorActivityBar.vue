<template>
  <nav class="ed-actbar">
    <button class="ed-act-item" :class="{ on: view === 'explorer' }" title="资源管理器" @click="setView('explorer')">
      <FolderFilled class="ed-act-ico" />
    </button>
    <button class="ed-act-item" :class="{ on: view === 'vars' }" title="变量" @click="setView('vars')">
      <VariableIcon :size="18" class="ed-act-ico" />
    </button>
    <button class="ed-act-item" :class="{ on: view === 'scm' }" title="源代码管理:相对最近版本的变更" @click="setView('scm')">
      <BranchesOutlined class="ed-act-ico" />
      <span v-if="badge > 0" class="ed-act-n">{{ badge > 99 ? '99+' : badge }}</span>
    </button>
    <div class="ed-act-gap"></div>
    <button class="ed-act-item" :class="{ on: aiOpen }" title="AI 助手" @click="$emit('toggle-ai')">
      <AiIcon :size="18" class="ed-act-ico" />
    </button>
  </nav>
</template>

<script setup>
import { computed } from 'vue'
import { FolderFilled, BranchesOutlined } from '@ant-design/icons-vue'
import VariableIcon from '@/components/icons/VariableIcon.vue'
import AiIcon from '@/components/icons/AiIcon.vue'

const props = defineProps({
  view: { type: String, default: 'explorer' },
  scmCount: { type: Number, default: 0 },
  aiOpen: { type: Boolean, default: false },
})
const emit = defineEmits(['update:view', 'toggle-ai'])
const badge = computed(() => props.scmCount)
const setView = (v) => emit('update:view', v)
</script>

<style scoped>
.ed-actbar { width: 44px; flex-shrink: 0; display: flex; flex-direction: column; align-items: center; gap: 4px; padding: 8px 0; background: var(--editor-panel-bg, #fff); border-right: 1px solid var(--editor-border, #e2e8f0); }
.ed-act-item { position: relative; width: 36px; height: 36px; border: none; background: transparent; border-radius: 8px; cursor: pointer; color: var(--color-text-secondary, #64748b); display: flex; align-items: center; justify-content: center; }
.ed-act-item:hover { background: var(--color-hover, #f1f5f9); color: var(--color-text, #1b1c1f); }
.ed-act-item.on { background: var(--color-nav-active, #eef0ec); color: var(--color-text, #1b1c1f); }
.ed-act-item.on::before { content: ''; position: absolute; left: -4px; top: 8px; bottom: 8px; width: 2.5px; border-radius: 2px; background: var(--color-brand, #16a34a); }
.ed-act-ico { font-size: 19px; }
.ed-act-n { position: absolute; right: -1px; top: -1px; min-width: 14px; height: 14px; padding: 0 3px; border-radius: 7px; background: #3e7bfa; color: #fff; font-size: 9px; display: flex; align-items: center; justify-content: center; }
.ed-act-gap { flex: 1; }
</style>
