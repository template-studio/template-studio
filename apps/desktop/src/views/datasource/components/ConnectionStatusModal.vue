<template>
  <a-modal
    v-model:open="modalVisible"
    :title="datasource ? `${datasource.name} - 连接状态` : '连接状态'"
    :footer="null"
    width="500px"
  >
    <a-spin :spinning="loading">
      <div v-if="statusData" class="status-content">
        <!-- 状态指示器 -->
        <div class="status-header">
          <span class="status-dot" :class="statusData.status"></span>
          <span class="status-text">
            {{ statusData.status === 'connected' ? '已连接' : '未连接' }}
          </span>
          <a-tag :color="statusData.status === 'connected' ? 'green' : 'red'">
            {{ statusData.type }}
          </a-tag>
        </div>

        <!-- 连接信息 -->
        <div class="status-grid">
          <div class="status-item" v-if="statusData.version">
            <span class="status-label">数据库版本</span>
            <span class="status-value">{{ statusData.version }}</span>
          </div>
          <div class="status-item" v-if="statusData.host">
            <span class="status-label">主机地址</span>
            <span class="status-value">{{ statusData.host }}:{{ statusData.port }}</span>
          </div>
          <div class="status-item" v-if="statusData.file">
            <span class="status-label">文件路径</span>
            <span class="status-value file-path">{{ statusData.file }}</span>
          </div>
          <div class="status-item" v-if="statusData.database">
            <span class="status-label">当前数据库</span>
            <span class="status-value">{{ statusData.database }}</span>
          </div>
          <div class="status-item">
            <span class="status-label">连接延迟</span>
            <span class="status-value">
              <span :class="{ 'latency-good': statusData.latency_ms < 50, 'latency-warn': statusData.latency_ms >= 50 }">
                {{ statusData.latency_ms }} ms
              </span>
            </span>
          </div>
          <div class="status-item" v-if="statusData.database_size">
            <span class="status-label">数据库大小</span>
            <span class="status-value">{{ statusData.database_size }}</span>
          </div>
          <div class="status-item" v-if="statusData.table_count !== undefined">
            <span class="status-label">表数量</span>
            <span class="status-value">{{ statusData.table_count }}</span>
          </div>
        </div>

        <!-- 连接池信息 -->
        <div class="status-section" v-if="statusData.pool_size !== undefined">
          <h4 class="section-title">连接池</h4>
          <div class="status-grid">
            <div class="status-item">
              <span class="status-label">池大小</span>
              <span class="status-value">{{ statusData.pool_size }}</span>
            </div>
            <div class="status-item">
              <span class="status-label">空闲连接</span>
              <span class="status-value">{{ statusData.pool_idle }}</span>
            </div>
            <div class="status-item" v-if="statusData.active_connections !== undefined">
              <span class="status-label">活跃连接</span>
              <span class="status-value">{{ statusData.active_connections }}</span>
            </div>
            <div class="status-item" v-if="statusData.max_connections">
              <span class="status-label">最大连接数</span>
              <span class="status-value">{{ statusData.max_connections }}</span>
            </div>
          </div>
        </div>

        <!-- 服务器信息 -->
        <div class="status-section" v-if="statusData.uptime_seconds">
          <h4 class="section-title">服务器</h4>
          <div class="status-grid">
            <div class="status-item">
              <span class="status-label">运行时间</span>
              <span class="status-value">{{ formatUptime(statusData.uptime_seconds) }}</span>
            </div>
          </div>
        </div>
      </div>
      <a-empty v-else-if="!loading" description="无法获取连接状态" />
    </a-spin>
  </a-modal>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { message } from 'ant-design-vue'
import * as datasourcesApi from '@/api/datasources'

const props = defineProps({
  open: { type: Boolean, default: false },
  datasource: { type: Object, default: null }
})

const emit = defineEmits(['update:open'])

const modalVisible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

const loading = ref(false)
const statusData = ref(null)

watch(() => props.open, async (val) => {
  if (val && props.datasource) {
    loading.value = true
    statusData.value = null
    try {
      statusData.value = await datasourcesApi.getConnectionStatus(props.datasource)
    } catch (error) {
      message.error('获取连接状态失败: ' + error)
    } finally {
      loading.value = false
    }
  }
})

const formatUptime = (seconds) => {
  if (!seconds) return '-'
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (days > 0) return `${days} 天 ${hours} 小时`
  if (hours > 0) return `${hours} 小时 ${minutes} 分钟`
  return `${minutes} 分钟`
}
</script>

<style scoped>
.status-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border);
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected {
  background: var(--color-success);
  box-shadow: 0 0 6px var(--color-success);
}

.status-dot.disconnected {
  background: var(--color-error);
}

.status-text {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text);
}

.status-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.status-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.status-label {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.status-value {
  font-size: 14px;
  color: var(--color-text);
  font-weight: 500;
}

.status-value.file-path {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  word-break: break-all;
}

.latency-good {
  color: var(--color-success);
}

.latency-warn {
  color: var(--color-warning);
}

.status-section {
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.section-title {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text);
}
</style>
