<template>
  <div class="analytics">
    <a-spin :spinning="loading">
      <!-- 页面标题 -->
      <div class="page-header">
        <h1>统计分析</h1>
        <p>深入了解模板系统的使用情况和性能指标</p>
      </div>

      <!-- 质量指标 -->
      <a-card title="质量指标" class="quality-metrics">
        <a-row :gutter="[20, 20]">
          <a-col :span="6">
            <a-statistic title="推荐模板" :value="overview.featuredTemplates">
              <template #prefix>
                <StarOutline style="color: #f0a020" />
              </template>
              <template #suffix>
                <span class="metric-suffix">/ {{ overview.totalTemplates }}</span>
              </template>
            </a-statistic>
          </a-col>
          <a-col :span="6">
            <a-statistic title="包含变量的模板" :value="overview.templatesWithVariables">
              <template #prefix>
                <CodeOutline style="color: #722ed1" />
              </template>
              <template #suffix>
                <span class="metric-suffix">/ {{ overview.totalTemplates }}</span>
              </template>
            </a-statistic>
          </a-col>
          <a-col :span="6">
            <a-statistic title="包含描述的模板" :value="overview.templatesWithDescription">
              <template #prefix>
                <DocumentOutline style="color: #52c41a" />
              </template>
              <template #suffix>
                <span class="metric-suffix">/ {{ overview.totalTemplates }}</span>
              </template>
            </a-statistic>
          </a-col>
          <a-col :span="6">
            <a-statistic title="平均文件数" :value="overview.avgFilesPerTemplate">
              <template #prefix>
                <FolderOutline style="color: #1890ff" />
              </template>
              <template #suffix>
                <span class="metric-suffix">/ 模板</span>
              </template>
            </a-statistic>
          </a-col>
        </a-row>
      </a-card>

      <!-- 详细图表分析 -->
      <div class="detailed-charts">
        <!-- 分类分析 -->
        <a-card title="分类详细分析" style="margin-bottom: 20px">
          <a-row :gutter="20">
            <a-col :span="12">
              <div class="chart-container">
                <h3>分类分布</h3>
                <v-chart class="chart" :option="categoryChartOption" />
              </div>
            </a-col>
            <a-col :span="12">
              <div class="category-table">
                <h3>分类统计表</h3>
                <a-table :data-source="categoryDistribution" :columns="categoryColumns" :pagination="false" row-key="categoryName" />
              </div>
            </a-col>
          </a-row>
        </a-card>

        <!-- 语言流行度分析 -->
        <a-card title="语言流行度分析" style="margin-bottom: 20px">
          <a-row :gutter="20">
            <a-col :span="12">
              <div class="chart-container">
                <h3>语言使用分布</h3>
                <v-chart class="chart" :option="languageChartOption" />
              </div>
            </a-col>
            <a-col :span="12">
              <div class="language-table">
                <h3>语言统计表</h3>
                <a-table :data-source="languagePopularity" :columns="languageColumns" :pagination="false" row-key="languageName" />
              </div>
            </a-col>
          </a-row>
        </a-card>

        <!-- 复杂度和趋势分析 -->
        <a-card title="模板复杂度和使用趋势">
          <a-row :gutter="20">
            <a-col :span="12">
              <div class="chart-container">
                <h3>模板复杂度分析</h3>
                <v-chart class="chart" :option="complexityChartOption" />
              </div>
            </a-col>
            <a-col :span="12">
              <div class="chart-container">
                <h3>创建趋势 (30天)</h3>
                <v-chart class="chart" :option="trendsChartOption" />
              </div>
            </a-col>
          </a-row>
        </a-card>
      </div>

      <!-- 操作区域 -->
      <a-card title="数据操作" class="actions-section">
        <a-space>
          <a-button type="primary" @click="refreshData">
            <template #icon>
              <RefreshOutline />
            </template>
            刷新数据
          </a-button>
          <a-button @click="exportData">
            <template #icon>
              <DownloadOutline />
            </template>
            导出报告
          </a-button>
          <a-select
            v-model:value="trendsRange"
            :options="trendsRangeOptions"
            style="width: 120px"
            @change="onTrendsRangeChange"
          />
        </a-space>
      </a-card>
    </a-spin>
  </div>
</template>

<script setup>
  import { ref, onMounted, computed, h } from 'vue';
  import { use } from 'echarts/core';
  import { PieChart, BarChart, LineChart } from 'echarts/charts';
  import {
    GridComponent,
    TooltipComponent,
    LegendComponent,
    TitleComponent,
  } from 'echarts/components';
  import { CanvasRenderer } from 'echarts/renderers';
  import VChart from 'vue-echarts';
  import { message } from 'ant-design-vue';
  import {
    StarOutline,
    CodeOutline,
    DocumentOutline,
    FolderOutline,
    RefreshOutline,
    DownloadOutline,
  } from '@/icons/ionicons5';
  import {
    getOverview,
    getCategoryDistribution,
    getLanguagePopularity,
    getTemplateComplexity,
    getUsageTrends,
  } from '@/api/statistics';

  // 注册ECharts组件
  use([
    PieChart,
    BarChart,
    LineChart,
    GridComponent,
    TooltipComponent,
    LegendComponent,
    TitleComponent,
    CanvasRenderer,
  ]);

  // 数据状态
  const loading = ref(true);
  const overview = ref({
    totalTemplates: 0,
    totalCategories: 0,
    totalLanguages: 0,
    totalFiles: 0,
    totalVariables: 0,
    featuredTemplates: 0,
    templatesWithVariables: 0,
    templatesWithDescription: 0,
    avgFilesPerTemplate: 0,
  });
  const categoryDistribution = ref([]);
  const languagePopularity = ref([]);
  const templateComplexity = ref({});
  const usageTrends = ref([]);
  const trendsRange = ref(30);

  // 趋势范围选项
  const trendsRangeOptions = [
    { label: '7天', value: 7 },
    { label: '30天', value: 30 },
    { label: '90天', value: 90 },
  ];

  // 表格列配置
  const categoryColumns = [
    { title: '分类名称', dataIndex: 'categoryName', key: 'categoryName' },
    { title: '模板数量', dataIndex: 'templateCount', key: 'templateCount' },
    {
      title: '占比',
      dataIndex: 'percentage',
      key: 'percentage',
      customRender: ({ text }) => h('span', {}, `${text}%`),
    },
  ];

  const languageColumns = [
    { title: '语言名称', dataIndex: 'languageName', key: 'languageName' },
    { title: '模板数量', dataIndex: 'templateCount', key: 'templateCount' },
    {
      title: '占比',
      dataIndex: 'percentage',
      key: 'percentage',
      customRender: ({ text }) => h('span', {}, `${text}%`),
    },
  ];

  // 图表配置 (复用Dashboard的配置)
  const categoryChartOption = computed(() => ({
    tooltip: {
      trigger: 'item',
      formatter: '{a} <br/>{b}: {c} ({d}%)',
    },
    legend: {
      bottom: '10%',
      left: 'center',
    },
    series: [
      {
        name: '分类分布',
        type: 'pie',
        radius: ['40%', '70%'],
        center: ['50%', '40%'],
        data: categoryDistribution.value.map((item) => ({
          value: item.templateCount,
          name: item.categoryName,
        })),
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
      },
    ],
  }));

  const languageChartOption = computed(() => ({
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'shadow',
      },
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      data: languagePopularity.value.map((item) => item.languageName),
      axisLabel: {
        rotate: 45,
      },
    },
    yAxis: {
      type: 'value',
    },
    series: [
      {
        name: '模板数量',
        type: 'bar',
        data: languagePopularity.value.map((item) => item.templateCount),
        itemStyle: {
          color: '#5470c6',
        },
      },
    ],
  }));

  const complexityChartOption = computed(() => ({
    tooltip: {
      trigger: 'item',
    },
    legend: {
      bottom: '5%',
      left: 'center',
    },
    series: [
      {
        name: '按文件数',
        type: 'pie',
        radius: [0, '30%'],
        center: ['25%', '40%'],
        data: [
          { value: templateComplexity.value.simpleTemplates || 0, name: '简单(1-3文件)' },
          { value: templateComplexity.value.mediumTemplates || 0, name: '中等(4-10文件)' },
          { value: templateComplexity.value.complexTemplates || 0, name: '复杂(10+文件)' },
        ],
      },
      {
        name: '按变量数',
        type: 'pie',
        radius: [0, '30%'],
        center: ['75%', '40%'],
        data: [
          { value: templateComplexity.value.noVariableTemplates || 0, name: '无变量' },
          { value: templateComplexity.value.fewVariableTemplates || 0, name: '少量变量(1-5)' },
          { value: templateComplexity.value.manyVariableTemplates || 0, name: '多变量(5+)' },
        ],
      },
    ],
  }));

  const trendsChartOption = computed(() => ({
    tooltip: {
      trigger: 'axis',
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true,
    },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: usageTrends.value.map((item) => item.date),
    },
    yAxis: {
      type: 'value',
    },
    series: [
      {
        name: '创建数量',
        type: 'line',
        stack: 'Total',
        data: usageTrends.value.map((item) => item.templateCreated),
        areaStyle: {},
        itemStyle: {
          color: '#91cc75',
        },
      },
    ],
  }));

  // 加载数据
  async function loadStatistics() {
    try {
      loading.value = true;

      const [overviewRes, categoryRes, languageRes, complexityRes, trendsRes] = await Promise.all([
        getOverview(),
        getCategoryDistribution(),
        getLanguagePopularity(),
        getTemplateComplexity(),
        getUsageTrends(trendsRange.value),
      ]);

      overview.value = overviewRes.data.data;
      categoryDistribution.value = categoryRes.data.data.items || [];
      languagePopularity.value = languageRes.data.data.items || [];
      templateComplexity.value = complexityRes.data.data;
      usageTrends.value = trendsRes.data.data.items || [];
    } catch (error) {
      console.error('加载统计数据失败:', error);
      message.error('加载统计数据失败');
    } finally {
      loading.value = false;
    }
  }

  // 刷新数据
  function refreshData() {
    loadStatistics();
    message.success('数据已刷新');
  }

  // 导出数据
  function exportData() {
    // 这里可以实现数据导出功能
    message.info('导出功能开发中...');
  }

  // 趋势范围变化
  async function onTrendsRangeChange(value) {
    try {
      const trendsRes = await getUsageTrends(value);
      usageTrends.value = trendsRes.data.data.items || [];
    } catch (error) {
      console.error('更新趋势数据失败:', error);
      message.error('更新趋势数据失败');
    }
  }

  // 生命周期
  onMounted(() => {
    loadStatistics();
  });
</script>

<style scoped>
  .analytics {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .page-header {
    text-align: center;
    margin-bottom: 24px;
  }

  .page-header h1 {
    margin: 0 0 8px 0;
    font-size: 28px;
    color: #333;
  }

  .page-header p {
    margin: 0;
    color: #666;
    font-size: 16px;
  }

  .quality-metrics {
    margin-bottom: 24px;
  }

  .metric-suffix {
    color: #999;
    font-size: 12px;
  }

  .detailed-charts {
    margin-top: 24px;
  }

  .chart-container h3,
  .category-table h3,
  .language-table h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
    color: #333;
  }

  .chart {
    height: 350px;
    width: 100%;
  }

  .actions-section {
    margin-top: 24px;
  }
</style>
