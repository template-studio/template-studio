<template>
  <div class="home-page-wrapper">
    <!-- 英雄区域 -->
    <div class="hero-section">
      <div class="hero-bg">
        <div class="hero-grid"></div>
        <div class="hero-glow hero-glow-1"></div>
        <div class="hero-glow hero-glow-2"></div>
      </div>
      <div class="hero-content">
        <div class="hero-badge">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
          <span>Template Studio</span>
        </div>
        <h1>
          从模板到代码<br>
          <span class="hero-highlight">一键生成项目</span>
        </h1>
        <p class="hero-subtitle">选择精心设计的项目模板，通过变量配置快速生成完整的项目结构，让开发效率提升 10 倍。</p>

        <div class="hero-actions">
          <button class="cta-primary" @click="scrollToTemplates">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>
            开始探索
          </button>
        </div>

        <!-- 统计数据 -->
        <div class="hero-stats">
          <div class="stat-item">
            <span class="stat-value">{{ statistics.totalTemplates || '100+' }}</span>
            <span class="stat-label">项目模板</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-value">{{ statistics.totalCategories || '10+' }}</span>
            <span class="stat-label">分类目录</span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-value">{{ statistics.totalLanguages || '20+' }}</span>
            <span class="stat-label">编程语言</span>
          </div>
        </div>
      </div>
    </div>

    <div class="home-page">
      <!-- 推荐模板 -->
      <div
        ref="templatesSection"
        class="featured-section"
        v-if="featuredTemplates && featuredTemplates.length > 0"
      >
        <div class="section-header">
          <h3 class="section-title">推荐模板</h3>
          <p class="section-subtitle">精选热门模板，助力快速开发</p>
        </div>

        <div class="container">
          <div v-if="loading" class="loading-container">
            <n-spin size="large" />
            <p>加载中...</p>
          </div>
          <div v-else class="templates-grid">
            <TemplateCard
              v-for="template in featuredTemplates"
              :key="template.id"
              :template="template"
              @click="useTemplate"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, onMounted } from 'vue';
  import { useRouter } from 'vue-router';
  import { getIndexData } from '@/api/indexData';
  import { useLanguageStore } from '@/store/modules/languageStore';
  import { useUser } from '@/store/modules/user';
  import TemplateCard from '@/components/TemplateCard.vue';

  const router = useRouter();
  const languageStore = useLanguageStore();
  const userStore = useUser();

  const loading = ref(false);
  const templatesSection = ref(null);
  const statistics = ref({
    totalTemplates: 0,
    totalCategories: 0,
    totalLanguages: 0,
    featuredCount: 0,
  });
  const categories = ref([]);
  const featuredTemplates = ref([]);

  const scrollToTemplates = () => {
    if (templatesSection.value) {
      templatesSection.value.scrollIntoView({ behavior: 'smooth' });
    }
  };

  const useTemplate = (template) => {
    router.push(`/template-generator/${template.id}`);
  };

  const fetchIndexData = async () => {
    loading.value = true;
    try {
      const response = await getIndexData({
        categoryLimit: 6,
        featuredLimit: 8,
      });

      if (response.data && response.data.data) {
        statistics.value = response.data.data.statistics || {};
        categories.value = response.data.data.categories || [];
        featuredTemplates.value = response.data.data.featuredTemplates || [];
      }
    } catch (error) {
    } finally {
      loading.value = false;
    }
  };

  onMounted(async () => {
    try {
      await languageStore.fetchLanguages();
      await fetchIndexData();
    } catch (error) {}
  });
</script>

<style scoped>
.home-page-wrapper {
  width: 100%;
  margin: 0;
  overflow-x: hidden;
}

.home-page {
  min-height: calc(100vh - 84px);
  background: #ffffff;
  position: relative;
  overflow-x: hidden;
  margin: 0;
  padding: 0;
}

/* ===== 英雄区域 ===== */
.hero-section {
  min-height: 85vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--client-hero-from, #0f172a), var(--client-hero-to, #1e293b));
  position: relative;
  z-index: 2;
  width: 100vw;
  margin-left: calc(-50vw + 50%);
  padding-top: 64px;
  overflow: hidden;
}

.hero-bg {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.hero-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(var(--client-theme-rgb), 0.04) 1px, transparent 1px),
    linear-gradient(90deg, rgba(var(--client-theme-rgb), 0.04) 1px, transparent 1px);
  background-size: 64px 64px;
}

.hero-glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(100px);
  pointer-events: none;
}

.hero-glow-1 {
  width: 500px;
  height: 500px;
  background: rgba(var(--client-theme-rgb), 0.08);
  top: -15%;
  right: -10%;
  animation: hero-drift 25s ease-in-out infinite;
}

.hero-glow-2 {
  width: 400px;
  height: 400px;
  background: rgba(59, 130, 246, 0.06);
  bottom: -10%;
  left: -5%;
  animation: hero-drift 30s ease-in-out infinite reverse;
}

@keyframes hero-drift {
  0%, 100% { transform: translate(0, 0); }
  25% { transform: translate(30px, -25px); }
  50% { transform: translate(-25px, 30px); }
  75% { transform: translate(20px, 15px); }
}

.hero-content {
  text-align: center;
  max-width: 640px;
  margin: 0 auto;
  padding: 0 24px;
  position: relative;
  z-index: 1;
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 20px;
  background: rgba(var(--client-theme-rgb), 0.1);
  border: 1px solid rgba(var(--client-theme-rgb), 0.2);
  color: var(--client-theme-color);
  font-size: 13px;
  font-weight: 500;
  margin-bottom: 32px;
  letter-spacing: 0.3px;
}

.hero-content h1 {
  font-size: 52px;
  font-weight: 700;
  margin: 0 0 24px 0;
  color: #f8fafc;
  letter-spacing: -1px;
  line-height: 1.2;
}

.hero-highlight {
  background: var(--client-theme-color);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hero-subtitle {
  font-size: 18px;
  color: #94a3b8;
  margin: 0 0 40px 0;
  line-height: 1.7;
  font-weight: 400;
}

/* ===== CTA 按钮 ===== */
.hero-actions {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  margin-bottom: 48px;
}

.cta-primary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 48px;
  padding: 0 28px;
  font-size: 15px;
  font-weight: 600;
  border-radius: 10px;
  background: var(--client-theme-color);
  border: none;
  color: #fff;
  cursor: pointer;
  transition: all 0.2s ease-out;
  letter-spacing: 0.3px;
}

.cta-primary:hover {
  background: var(--client-theme-dark);
  transform: translateY(-1px);
  box-shadow: 0 8px 24px rgba(var(--client-theme-rgb), 0.3);
}

.cta-primary:active {
  transform: translateY(0);
}

/* ===== 统计数据 ===== */
.hero-stats {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 32px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #f8fafc;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.stat-label {
  font-size: 13px;
  color: #64748b;
  font-weight: 400;
}

.stat-divider {
  width: 1px;
  height: 32px;
  background: rgba(255, 255, 255, 0.08);
}

/* ===== 推荐模板区域 ===== */
.featured-section {
  position: relative;
  z-index: 2;
  padding: 80px 0;
  background: #f8fafc;
}

.section-header {
  text-align: center;
  margin-bottom: 48px;
}

.section-title {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 12px 0;
  color: #0f172a;
  letter-spacing: -0.5px;
}

.section-subtitle {
  font-size: 15px;
  color: #64748b;
  margin: 0;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 0;
  color: #64748b;
  text-align: center;
}

.loading-container p {
  margin-top: 16px;
  font-size: 14px;
}

.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
}

/* ===== 响应式 ===== */
@media (max-width: 768px) {
  .hero-section {
    min-height: 80vh;
    padding: 40px 20px;
  }

  .hero-content h1 {
    font-size: 32px;
    letter-spacing: -0.5px;
  }

  .hero-subtitle {
    font-size: 16px;
    margin-bottom: 32px;
  }

  .hero-actions {
    flex-direction: column;
    gap: 12px;
  }

  .cta-primary {
    width: 100%;
    max-width: 280px;
    justify-content: center;
  }

  .hero-stats {
    gap: 20px;
  }

  .stat-value {
    font-size: 20px;
  }

  .section-title {
    font-size: 22px;
  }

  .templates-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }
}

/* ===== prefers-reduced-motion ===== */
@media (prefers-reduced-motion: reduce) {
  .hero-glow { animation: none; }
  .cta-primary { transition: none; }
}
</style>
