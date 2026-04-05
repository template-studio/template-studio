<template>
  <div class="not-found">
    <div class="not-found-content">
      <div class="error-code">404</div>
      <h1 class="error-title">页面未找到</h1>
      <p class="error-desc">抱歉，您访问的页面不存在或已被移除</p>
      <div class="error-actions">
        <a-button type="primary" size="large" @click="goHome">
          <template #icon><HomeOutlined /></template>
          返回首页
        </a-button>
        <a-button size="large" @click="goBack">
          <template #icon><ArrowLeftOutlined /></template>
          返回上一页
        </a-button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { useRouter } from 'vue-router'
import { HomeOutlined, ArrowLeftOutlined } from '@ant-design/icons-vue'

const router = useRouter()

const goHome = () => {
  router.push('/home')
}

const goBack = () => {
  router.back()
}
</script>

<style scoped>
.not-found {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background);
}

.not-found-content {
  text-align: center;
  padding: 40px;
}

.error-code {
  font-size: 120px;
  font-weight: 700;
  color: var(--color-primary);
  line-height: 1;
  margin-bottom: 16px;
  opacity: 0.3;
}

.error-title {
  font-size: 28px;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 12px;
}

.error-desc {
  font-size: 16px;
  color: var(--color-text-secondary);
  margin: 0 0 32px;
}

.error-actions {
  display: flex;
  gap: 16px;
  justify-content: center;
}
</style>
