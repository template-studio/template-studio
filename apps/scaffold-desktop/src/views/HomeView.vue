<template>
  <div class="home-view">
    <div class="welcome-section">
      <h1>欢迎使用 Template Studio</h1>
      <p>一个美观、可扩展的模板化开发平台</p>
    </div>

    <div class="features-grid">
      <div class="feature-card">
        <h3>🚀 现代化技术栈</h3>
        <p>基于 Vue 3、Tauri 和 Ant Design Vue 构建</p>
      </div>
      <div class="feature-card">
        <h3>🎨 精美界面</h3>
        <p>响应式设计，支持明暗主题切换</p>
      </div>
      <div class="feature-card">
        <h3>⚡ 高性能</h3>
        <p>为速度和内存效率进行了优化</p>
      </div>
      <div class="feature-card">
        <h3>🛡️ 安全可靠</h3>
        <p>遵循 Tauri 安全最佳实践</p>
      </div>
    </div>
  </div>
</template>

<script setup>
// Home page component
</script>

<style scoped>
.home-view {
  padding: var(--spacing-lg);
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
  /* 添加背景色，继承全局主题 */
  background: var(--color-background);
  color: var(--color-text);
}

.welcome-section {
  text-align: center;
  margin-bottom: var(--spacing-xl);
}

.welcome-section h1 {
  font-size: 2.5rem;
  margin-bottom: var(--spacing-md);
  color: var(--color-primary);
}

.welcome-section p {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: var(--spacing-lg);
  margin-top: var(--spacing-xl);
}

.feature-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
  padding: var(--spacing-lg);
  text-align: center;
  transition: transform var(--transition-fast), box-shadow var(--transition-fast);
}

.feature-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.feature-card h3 {
  margin-bottom: var(--spacing-sm);
  color: var(--color-text);
}

.feature-card p {
  color: var(--color-text-secondary);
  line-height: 1.6;
}
</style>