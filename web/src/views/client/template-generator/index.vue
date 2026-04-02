<template>
  <div class="template-generator-wrapper">
    <!-- 固定顶部导航 -->
    <div class="generator-header">
      <div class="header-content">
        <div class="header-left">
          <div class="back-btn" @click="goBack">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5"/><polyline points="12 19 5 12 12 5"/></svg>
            <span>返回</span>
          </div>
          <div class="template-title">
            <span class="title-text">使用模板</span>
            <span class="title-divider">/</span>
            <span class="template-name" v-if="templateInfo">{{ templateInfo.name }}</span>
          </div>
        </div>

        <!-- 步骤指示器 -->
        <div class="step-indicator">
          <div
            v-for="(step, index) in steps"
            :key="index"
            class="step-item"
            :class="{
              active: currentStep === index + 1,
              completed: currentStep > index + 1,
            }"
          >
            <div class="step-dot">
              <svg v-if="currentStep > index + 1" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
              <span v-else>{{ index + 1 }}</span>
            </div>
            <span class="step-label">{{ step.label }}</span>
            <div v-if="index < steps.length - 1" class="step-line" :class="{ filled: currentStep > index + 1 }"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="template-generator">
      <div class="generator-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-container">
          <n-spin size="large" />
          <p>正在加载模板信息...</p>
        </div>

        <!-- 步骤1: 模板介绍 -->
        <div v-else-if="currentStep === 1" class="step-content">
          <StepIntro
            :template-info="templateInfo"
            :version-list="versionList"
            :selected-version="selectedVersion"
            @update-version="updateVersion"
            @next="nextStep"
          />
        </div>

        <!-- 步骤2: 变量配置 -->
        <div v-else-if="currentStep === 2" class="step-content">
          <StepVariables
            :template-info="templateInfo"
            :selected-version="selectedVersion"
            :variables="variables"
            @prev="prevStep"
            @next="nextStep"
            @update-variables="updateVariables"
          />
        </div>

        <!-- 步骤3: 预览确认 -->
        <div v-else-if="currentStep === 3" class="step-content">
          <StepPreview
            :template-info="templateInfo"
            :selected-version="selectedVersion"
            :variables="variables"
            @prev="prevStep"
            @generate="generateProject"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
  import { ref, onMounted } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import { useMessage } from 'naive-ui';
  import { getTemplateDetail } from '@/api/templates';
  import { listReleases } from '@/api/releases';
  import StepIntro from './components/StepIntro.vue';
  import StepVariables from './components/StepVariables.vue';
  import StepPreview from './components/StepPreview.vue';

  const route = useRoute();
  const router = useRouter();
  const message = useMessage();

  const steps = [
    { label: '模板介绍', key: 'intro' },
    { label: '配置变量', key: 'variables' },
    { label: '预览确认', key: 'preview' },
  ];

  const currentStep = ref(1);
  const templateInfo = ref(null);
  const variables = ref({});
  const loading = ref(false);

  const versionList = ref([]);
  const selectedVersion = ref('');

  const loadTemplateInfo = async () => {
    if (!route.params.id) {
      message.error('缺少模板ID参数');
      return;
    }

    loading.value = true;
    try {
      const res = await getTemplateDetail({ id: route.params.id });
      templateInfo.value = res.data.data;
    } catch (error) {
      message.error('加载模板信息失败');
    } finally {
      loading.value = false;
    }
  };

  const loadVersions = async () => {
    if (!route.params.id) return;
    try {
      const res = await listReleases(route.params.id);
      const data = res.data;
      if (data.code === 0) {
        versionList.value = data.data.versions || [];
      }
    } catch (error) {
      console.error('加载版本列表失败:', error);
    }
  };

  const nextStep = () => {
    if (currentStep.value < 3) currentStep.value++;
  };

  const prevStep = () => {
    if (currentStep.value > 1) currentStep.value--;
  };

  const updateVariables = (newVariables) => {
    variables.value = { ...newVariables };
  };

  const updateVersion = (newVersion) => {
    selectedVersion.value = newVersion;
    variables.value = {};
  };

  const generateProject = async () => {
    console.warn('generateProject函数已迁移至StepPreview组件');
  };

  const goBack = () => {
    router.back();
  };

  onMounted(async () => {
    if (!route.params.id) {
      message.warning('请先选择一个模板');
      router.push('/templates');
      return;
    }
    await loadTemplateInfo();
    await loadVersions();
  });
</script>

<style>
  .main-content {
    padding-bottom: 0 !important;
  }
  .footer-bar {
    margin-top: 0 !important;
    padding: 16px 0 !important;
  }
</style>

<style scoped>
  .template-generator-wrapper {
    min-height: 100vh;
    padding-top: 60px;
    background: #f8fafc;
  }

  /* ===== Header ===== */
  .generator-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background: #fff;
    border-bottom: 1px solid #e2e8f0;
    z-index: 1000;
  }

  .header-content {
    margin: 0;
    padding: 0 32px;
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 1400px;
    margin: 0 auto;
    box-sizing: border-box;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    color: #64748b;
    font-size: 14px;
    cursor: pointer;
    padding: 6px 10px;
    border-radius: 6px;
    transition: all 0.2s ease;
  }

  .back-btn:hover {
    color: #0f172a;
    background: #f1f5f9;
  }

  .template-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title-text {
    font-size: 14px;
    color: #94a3b8;
  }

  .title-divider {
    color: #e2e8f0;
  }

  .template-name {
    font-size: 15px;
    font-weight: 600;
    color: #0f172a;
  }

  /* ===== Steps ===== */
  .step-indicator {
    display: flex;
    align-items: center;
    gap: 0;
  }

  .step-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .step-dot {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: #f1f5f9;
    color: #94a3b8;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 600;
    transition: all 0.25s ease;
    flex-shrink: 0;
  }

  .step-item.active .step-dot {
    background: var(--client-theme-color);
    color: #fff;
    box-shadow: 0 2px 8px rgba(var(--client-theme-rgb), 0.3);
  }

  .step-item.completed .step-dot {
    background: var(--client-theme-color);
    color: #fff;
  }

  .step-label {
    font-size: 13px;
    color: #94a3b8;
    font-weight: 500;
    transition: color 0.25s ease;
    white-space: nowrap;
  }

  .step-item.active .step-label {
    color: #0f172a;
    font-weight: 600;
  }

  .step-item.completed .step-label {
    color: var(--client-theme-color);
  }

  .step-line {
    width: 40px;
    height: 2px;
    background: #e2e8f0;
    margin: 0 12px;
    border-radius: 1px;
    transition: background 0.25s ease;
  }

  .step-line.filled {
    background: var(--client-theme-color);
  }

  /* ===== Content ===== */
  .generator-content {
    padding: 24px 32px;
    max-width: 1400px;
    margin: 0 auto;
    box-sizing: border-box;
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 400px;
    gap: 16px;
    color: #64748b;
  }

  .loading-container p {
    font-size: 14px;
  }

  .step-content {
    background: #fff;
    border-radius: 12px;
    border: 1px solid #e2e8f0;
    min-height: calc(100vh - 120px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
