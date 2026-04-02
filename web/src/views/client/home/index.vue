<template>
  <div class="home-page-wrapper">
    <!-- 英雄区域 - 完全独立，不受Container限制 -->
    <div class="hero-section">
      <div class="hero-content">
        <h1>Template Studio</h1>
        <p class="hero-subtitle">从数百个精心设计的项目模板中选择，一键生成完整项目结构</p>

        <div class="hero-actions">
          <n-button type="primary" size="large" class="cta-button" @click="scrollToTemplates">
            开始探索
          </n-button>
          <router-link to="/admin" class="admin-link">
            <n-icon class="admin-icon">
              <SettingsOutline />
            </n-icon>
            <span class="admin-text">后台管理</span>
          </router-link>
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
          <div class="section-content">
            <h3 class="section-title">
              <span class="section-icon">✨</span>
              推荐模板
            </h3>
            <p class="section-subtitle">精选热门模板，助力快速开发</p>
          </div>
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
  import TemplateCard from '@/components/TemplateCard.vue';
  import { SettingsOutline } from '@vicons/ionicons5';

  const router = useRouter();
  const languageStore = useLanguageStore();

  // 响应式数据
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

  // 滚动到模板区域
  const scrollToTemplates = () => {
    if (templatesSection.value) {
      templatesSection.value.scrollIntoView({ behavior: 'smooth' });
    }
  };

  // 使用模板
  const useTemplate = (template) => {
    router.push(`/template-generator/${template.id}`);
  };

  // 获取首页数据
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

  // 页面加载时获取数据
  onMounted(async () => {
    try {
      // 先获取语言列表
      await languageStore.fetchLanguages();
      // 再获取首页数据
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
    /* 确保可以突破Container限制 */
    margin: 0;
    padding: 0;
  }

  /* 背景装饰 */
  .bg-decoration {
    position: fixed;
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    pointer-events: none;
    z-index: 1;
  }

  .circle {
    position: absolute;
    border-radius: 50%;
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.1), rgba(139, 92, 246, 0.1));
    backdrop-filter: blur(10px);
    animation: float 8s ease-in-out infinite;
  }

  .circle-1 {
    width: 400px;
    height: 400px;
    top: -200px;
    right: -200px;
    animation-delay: 0s;
  }

  .circle-2 {
    width: 300px;
    height: 300px;
    top: 30%;
    left: -150px;
    animation-delay: 2s;
  }

  .circle-3 {
    width: 250px;
    height: 250px;
    bottom: 20%;
    right: 10%;
    animation-delay: 4s;
  }

  .circle-4 {
    width: 200px;
    height: 200px;
    bottom: -100px;
    left: 20%;
    animation-delay: 6s;
  }

  @keyframes float {
    0%,
    100% {
      transform: translateY(0px) rotate(0deg);
    }
    50% {
      transform: translateY(-30px) rotate(180deg);
    }
  }

  /* 英雄区域 - 完全独立，占满全屏 */
  .hero-section {
    min-height: 85vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
    position: relative;
    z-index: 2;
    /* 占满整个视口宽度 */
    width: 100vw;
    /* 使用负margin将元素拉到视口的最左边 */
    margin-left: calc(-50vw + 50%);
    /* 处理导航栏高度 */
    margin-top: 0;
    padding-top: 64px;
  }

  .hero-content {
    text-align: center;
    color: #ffffff;
    max-width: 700px;
    margin: 0 auto;
    padding: 0 20px;
    position: relative;
    z-index: 1;
  }

  .hero-content h1 {
    font-size: 48px;
    font-weight: 500;
    margin: 0 0 24px 0;
    color: #ffffff;
    letter-spacing: -0.5px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }

  .hero-subtitle {
    font-size: 20px;
    color: rgba(255, 255, 255, 0.9);
    margin: 0 0 48px 0;
    line-height: 1.6;
    font-weight: 400;
  }

  /* 行动按钮 */
  .hero-actions {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
  }

  .cta-button {
    height: 48px;
    padding: 0 32px;
    font-size: 15px;
    font-weight: 500;
    border-radius: 24px;
    background: #ffffff;
    border: none;
    color: #4285f4;
    transition: all 0.2s ease;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .cta-button:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  /* 英雄区域后台管理链接样式 */
  .admin-link {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border-radius: 24px;
    background: rgba(255, 255, 255, 0.15);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.3);
    color: rgba(255, 255, 255, 0.9);
    text-decoration: none;
    font-weight: 500;
    font-size: 13px;
    transition: all 0.3s ease;
  }

  .admin-link:hover {
    background: rgba(255, 255, 255, 0.25);
    border-color: rgba(255, 255, 255, 0.5);
    color: #ffffff;
    text-decoration: none;
    transform: translateY(-1px);
    box-shadow: 0 4px 16px rgba(255, 255, 255, 0.1);
  }

  .admin-link:active {
    transform: translateY(0);
    box-shadow: 0 2px 8px rgba(255, 255, 255, 0.05);
  }

  .admin-icon {
    font-size: 14px;
    transition: transform 0.2s ease;
  }

  .admin-link:hover .admin-icon {
    transform: rotate(90deg);
  }

  .admin-text {
    letter-spacing: 0.2px;
    white-space: nowrap;
  }

  /* 区域通用样式 */
  .featured-section,
  .categories-section {
    position: relative;
    z-index: 2;
    padding: 80px 0;
  }

  .featured-section {
    background: #ffffff;
  }

  .categories-section {
    background: #f8fafc;
  }

  .section-header {
    text-align: center;
    margin-bottom: 60px;
  }

  .section-content {
    max-width: 600px;
    margin: 0 auto;
  }

  .section-title {
    font-size: 32px;
    font-weight: 500;
    margin: 0 0 16px 0;
    color: #202124;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }

  .section-icon {
    font-size: 36px;
  }

  .section-subtitle {
    font-size: 16px;
    color: #5f6368;
    margin: 0;
    font-weight: 400;
  }

  /* 容器 */
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
  }

  /* 加载和空状态 */
  .loading-container,
  .empty-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 0;
    color: #6b7280;
    text-align: center;
  }

  .loading-container p,
  .empty-container p {
    margin-top: 16px;
    font-size: 16px;
  }

  /* 分类块 */
  .category-block {
    margin-bottom: 80px;
  }

  .category-block:last-child {
    margin-bottom: 0;
  }

  .category-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 40px;
    padding-bottom: 20px;
    border-bottom: 2px solid #f1f5f9;
  }

  .category-info {
    flex: 1;
  }

  .category-title {
    font-size: 24px;
    font-weight: 500;
    margin: 0 0 8px 0;
    color: #202124;
  }

  .category-description {
    font-size: 16px;
    color: #5f6368;
    margin: 0;
  }

  .category-stats {
    margin-left: 20px;
  }

  .template-count {
    background: linear-gradient(135deg, #4285f4, #34a853);
    color: white;
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 14px;
    font-weight: 500;
  }

  /* 模板网格 */
  .templates-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 24px;
  }

  @media (max-width: 1200px) {
    .templates-grid {
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    }
  }

  @media (max-width: 768px) {
    .templates-grid {
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 16px;
    }
  }

  /* 模板卡片 */
  .template-card {
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(12px);
    border-radius: 16px;
    box-shadow: 0 1px 2px 0 rgba(60, 64, 67, 0.1), 0 1px 3px 1px rgba(60, 64, 67, 0.05);
    overflow: hidden;
    transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    cursor: pointer;
    position: relative;
    border: 1px solid rgba(255, 255, 255, 0.6);
    will-change: transform;
    backface-visibility: hidden;
  }

  .template-card:hover {
    transform: translateY(-8px) scale(1.02);
    box-shadow: 0 12px 40px rgba(66, 133, 244, 0.15);
    border-color: rgba(66, 133, 244, 0.3);
    background: rgba(255, 255, 255, 0.95);
  }

  /* 上方视觉区域 */
  .card-visual-area {
    width: 100%;
    height: 180px;
    position: relative;
    overflow: hidden;
  }

  .visual-bg {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  /* Shimmer光泽效果 */
  .visual-bg::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      45deg,
      transparent 30%,
      rgba(255, 255, 255, 0.2) 50%,
      transparent 70%
    );
    animation: shimmer 3s infinite;
  }

  @keyframes shimmer {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(100%);
    }
  }

  .code-snippet-preview {
    font-family: 'Courier New', 'Consolas', 'Monaco', monospace;
    font-size: 10px;
    line-height: 1.4;
    color: rgba(255, 255, 255, 0.4);
    white-space: pre;
    overflow: hidden;
    padding: 20px;
    text-align: left;
    position: relative;
    z-index: 1;
  }

  /* 卡片内容区域 */
  .card-content-area {
    padding: 20px;
    background: white;
  }

  .template-badge {
    position: absolute;
    top: 12px;
    right: 12px;
    background: rgba(255, 255, 255, 0.95);
    color: #34a853;
    font-size: 11px;
    font-weight: 700;
    padding: 6px 14px;
    border-radius: 20px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    transition: all 0.3s ease;
    z-index: 2;
    backdrop-filter: blur(10px);
  }

  .template-card:hover .template-badge {
    transform: scale(1.08);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .template-name {
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 10px 0;
    color: #202124;
    line-height: 1.3;
    letter-spacing: -0.3px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .template-card:hover .template-name {
    color: #1967d2;
  }

  .template-description {
    font-size: 13px;
    color: #5f6368;
    margin: 0 0 16px 0;
    line-height: 1.6;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .template-languages {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }

  /* 卡片底部 */
  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 16px;
    border-top: 1px solid rgba(0, 0, 0, 0.06);
  }

  .card-author {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .author-avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: linear-gradient(135deg, #4285f4 0%, #34a853 100%);
    flex-shrink: 0;
  }

  .author-name {
    font-size: 13px;
    color: #5f6368;
    font-weight: 500;
  }

  .card-stats {
    display: flex;
    gap: 12px;
    font-size: 13px;
    color: #80868b;
  }

  .stat-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .stat-icon {
    font-size: 14px;
  }

  .template-languages :deep(.n-tag) {
    background: #f8f9fa;
    border: 1px solid #e8eaed;
    color: #5f6368;
    font-size: 12px;
    padding: 4px 12px;
    border-radius: 12px;
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .template-card:hover .template-languages :deep(.n-tag) {
    background: linear-gradient(135deg, #e8f0fe 0%, #e6f4ea 100%);
    border-color: #d2e3fc;
    color: #1967d2;
    transform: translateY(-2px);
  }

  /* 响应式设计 */
  @media (max-width: 768px) {
    .hero-section {
      min-height: 80vh;
      padding: 40px 20px;
    }

    .hero-content h1 {
      font-size: 32px;
      flex-direction: column;
      gap: 8px;
    }

    .hero-subtitle {
      font-size: 18px;
      margin-bottom: 32px;
    }

    .hero-actions {
      flex-direction: column;
      gap: 12px;
    }

    .cta-button {
      width: 100%;
      max-width: 280px;
    }

    .section-title {
      font-size: 24px;
      flex-direction: column;
      gap: 8px;
    }

    .category-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .category-stats {
      margin-left: 0;
    }

    .templates-grid {
      grid-template-columns: 1fr;
    }

    .template-card {
      max-width: 100%;
    }
  }
</style>
