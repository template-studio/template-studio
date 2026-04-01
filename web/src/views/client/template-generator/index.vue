<template>
  <div class="template-generator-wrapper">
    <!-- 固定顶部导航 -->
    <div class="generator-header">
      <div class="header-content">
        <div class="header-left">
          <n-button text @click="goBack">
            <template #icon>
              <n-icon><ArrowBack /></n-icon>
            </template>
            返回
          </n-button>
          <div class="template-title">
            <span class="title-text">使用模板</span>
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
            <div class="step-number">{{ index + 1 }}</div>
            <div class="step-label">{{ step.label }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="template-generator">
      <!-- 步骤内容 -->
      <div class="generator-content">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-container">
          <n-spin size="large">
            <template #description> 正在加载模板信息... </template>
          </n-spin>
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
  import { ArrowBack } from '@vicons/ionicons5';
  import { getTemplateDetail } from '@/api/templates';
  import { listReleases } from '@/api/releases';
  import StepIntro from './components/StepIntro.vue';
  import StepVariables from './components/StepVariables.vue';
  import StepPreview from './components/StepPreview.vue';

  const route = useRoute();
  const router = useRouter();
  const message = useMessage();

  // 步骤配置
  const steps = [
    { label: '模板介绍', key: 'intro' },
    { label: '配置变量', key: 'variables' },
    { label: '预览确认', key: 'preview' },
  ];

  // 状态
  const currentStep = ref(1);
  const templateInfo = ref(null);
  const variables = ref({});
  const loading = ref(false);

  // 版本管理
  const versionList = ref([]);
  const selectedVersion = ref(''); // 空字符串表示最新版本

  // 获取模板信息
  const loadTemplateInfo = async () => {
    if (!route.params.id) {
      message.error('缺少模板ID参数');
      return;
    }

    loading.value = true;
    try {
      const res = await getTemplateDetail({ id: route.params.id });
      templateInfo.value = res.data.data; // 修复：两层 data，不是三层
      console.log('模板信息加载成功:', templateInfo.value);
    } catch (error) {
      message.error('加载模板信息失败');
      console.error(error);
    } finally {
      loading.value = false;
    }
  };

  // 加载版本列表
  const loadVersions = async () => {
    if (!route.params.id) {
      return;
    }

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

  // 步骤导航
  const nextStep = () => {
    if (currentStep.value < 3) {
      currentStep.value++;
    }
  };

  const prevStep = () => {
    if (currentStep.value > 1) {
      currentStep.value--;
    }
  };

  // 更新变量
  const updateVariables = (newVariables) => {
    variables.value = { ...newVariables };
  };

  // 更新版本
  const updateVersion = (newVersion) => {
    selectedVersion.value = newVersion;
    // 清空变量，因为不同版本的变量定义可能不同
    variables.value = {};
    console.log('版本已更新为:', newVersion || 'Latest');
  };

  // 生成项目 - 实际生成功能在StepPreview组件中实现
  const generateProject = async () => {
    // 注意：实际的生成和下载功能已移至StepPreview.vue组件中
    // 此函数仅为兼容性保留，实际不会被调用
    console.warn('generateProject函数已迁移至StepPreview组件');
  };

  // 返回上一页
  const goBack = () => {
    router.back();
  };

  onMounted(async () => {
    // 如果没有模板ID，重定向到模板列表页
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
  /* 全局样式覆盖 - 不使用scoped */
  .main-content {
    padding-bottom: 0 !important;
  }

  .footer-bar {
    margin-top: 0 !important;
    padding: 16px 0 !important;
  }
</style>

<style scoped>
  /* 外层容器 */
  .template-generator-wrapper {
    min-height: 100vh;
    padding-top: 72px; /* 为固定的header留出空间 */
  }

  /* 固定顶部导航 */
  .generator-header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    background: #fff;
    border-bottom: 1px solid #e0e0e0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.03);
    z-index: 1000;
  }

  .header-content {
    margin: 0;
    padding: 16px 30px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    box-sizing: border-box;
  }

  /* 主要内容区域 */
  .template-generator {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .template-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .title-text {
    font-size: 16px;
    color: #666;
  }

  .template-name {
    font-size: 18px;
    font-weight: bold;
    color: #18a058;
  }

  .step-indicator {
    display: flex;
    align-items: center;
    gap: 32px;
  }

  .step-item {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: all 0.3s;
  }

  .step-number {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #f0f0f0;
    color: #999;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    transition: all 0.3s;
  }

  .step-label {
    font-size: 14px;
    color: #666;
    transition: all 0.3s;
  }

  .step-item.active .step-number {
    background: #18a058;
    color: #fff;
  }

  .step-item.active .step-label {
    color: #18a058;
    font-weight: bold;
  }

  .step-item.completed .step-number {
    background: #52c41a;
    color: #fff;
  }

  .step-item.completed .step-label {
    color: #52c41a;
  }

  .generator-content {
    flex: 1;
    padding: 30px;
    width: 100%;
    box-sizing: border-box;
    overflow-y: auto;
  }

  .loading-container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 400px;
  }

  .step-content {
    background: #fff;
    border-radius: 12px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
    min-height: calc(100vh - 132px);
    display: flex;
    flex-direction: column;
  }
</style>
