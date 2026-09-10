<template>
  <a-modal :open="open" title="从项目转换" :footer="null" :width="560" @cancel="$emit('update:open', false)">
    <div class="csm">
      <div class="csm-sub">完整克隆到本地镜像后操作,原始仓库只读;转换基于已提交内容(HEAD)</div>
      <div class="csm-mode">
        <div class="csm-mode-item" :class="{ active: mode === 'remote' }" @click="mode = 'remote'">
          <span class="csm-mode-name"><GithubOutlined /> 远程仓库</span>
          <span class="csm-mode-desc">GitHub / Gitee / GitLab</span>
        </div>
        <div class="csm-mode-item" :class="{ active: mode === 'local' }" @click="mode = 'local'">
          <span class="csm-mode-name"><FolderOutlined /> 本地仓库</span>
          <span class="csm-mode-desc">本机已提交的 git 项目</span>
        </div>
      </div>

      <template v-if="mode === 'remote'">
        <a-input v-model:value="remote" placeholder="https://github.com/user/repo.git" size="large" class="csm-mono" allow-clear @pressEnter="submit" />
        <a-input v-model:value="branch" placeholder="分支(可选,默认主分支)" allow-clear @pressEnter="submit" />
      </template>
      <template v-else>
        <div class="csm-local-row">
          <a-input v-model:value="local" placeholder="本地 git 仓库路径,如 D:\projects\my-app" size="large" class="csm-mono" @pressEnter="submit" />
          <a-button size="large" @click="pickLocalDir">
            <template #icon><FolderOpenOutlined /></template>
            浏览
          </a-button>
        </div>
        <div class="csm-tip">要求目录内已 git init 且有提交;只读克隆,不改动原始目录</div>
      </template>

      <div v-if="error" class="csm-error">{{ error }}</div>
      <div class="csm-actions">
        <a-button @click="$emit('update:open', false)">取消</a-button>
        <a-button type="primary" @click="submit">开始转换</a-button>
      </div>
    </div>
  </a-modal>
</template>

<script setup>
import { ref, watch } from 'vue'
import { message } from 'ant-design-vue'
import { GithubOutlined, FolderOutlined, FolderOpenOutlined } from '@ant-design/icons-vue'

const props = defineProps({ open: { type: Boolean, default: false } })
const emit = defineEmits(['update:open', 'start'])

const mode = ref('remote')
const remote = ref('')
const local = ref('')
const branch = ref('')
const error = ref('')

// 每次打开重置为远程模式,清空上次的错误
watch(() => props.open, (v) => { if (v) error.value = '' })

const pickLocalDir = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const sel = await open({ directory: true, multiple: false, title: '选择本地 git 仓库' })
    if (sel) local.value = sel
  } catch (e) {
    message.error('选择目录失败: ' + (e.message || e))
  }
}

const submit = () => {
  const isRemote = mode.value === 'remote'
  const src = (isRemote ? remote.value : local.value).trim()
  if (!src) {
    error.value = isRemote ? '请输入远程仓库 URL' : '请选择或输入本地 git 仓库路径'
    return
  }
  if (isRemote && !/^(https?:\/\/|git@|ssh:\/\/)/.test(src)) {
    error.value = '远程仓库需以 http(s):// 或 git@ 开头;本地项目请切换到「本地仓库」模式'
    return
  }
  error.value = ''
  emit('start', { source: src, branch: isRemote ? (branch.value.trim() || null) : null })
}
</script>

<style scoped>
.csm { display: flex; flex-direction: column; gap: 12px; padding: 8px 4px 4px; }
.csm-sub { font-size: 12px; color: var(--color-text-secondary, #999); }
.csm-mode { display: flex; gap: 8px; }
.csm-mode-item { flex: 1; display: flex; flex-direction: column; gap: 2px; padding: 10px 12px; border: 1px solid var(--color-border, #e5e5e2); border-radius: 8px; cursor: pointer; transition: border-color 0.15s, background 0.15s; }
.csm-mode-item:hover { border-color: var(--color-text-secondary, #bbb); }
.csm-mode-item.active { border-color: var(--color-brand, #16a34a); background: rgba(22, 163, 74, 0.05); }
.csm-mode-name { display: flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; color: var(--color-text, #1b1c1f); }
.csm-mode-desc { font-size: 11px; color: var(--color-text-secondary, #999); padding-left: 20px; }
.csm-local-row { display: flex; gap: 8px; }
.csm-tip { font-size: 12px; color: var(--color-text-secondary, #999); }
.csm-error { color: #dc2626; font-size: 12px; white-space: pre-wrap; }
.csm-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
.csm-mono { font-family: Consolas, 'JetBrains Mono', monospace; }
</style>
