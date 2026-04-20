<template>
  <div class="dashboard">
    <a-spin :spinning="loading">
      <!-- 基础统计卡片 -->
      <a-row :gutter="[20, 20]">
        <a-col :span="6">
          <a-card>
            <a-statistic title="模板总数" :value="overview.totalTemplates">
              <template #prefix>
                <DocumentTextOutline style="color: #18a058" />
              </template>
            </a-statistic>
          </a-card>
        </a-col>
        <a-col :span="6">
          <a-card>
            <a-statistic title="分类数量" :value="overview.totalCategories">
              <template #prefix>
                <LayersOutline style="color: #2080f0" />
              </template>
            </a-statistic>
          </a-card>
        </a-col>
        <a-col :span="6">
          <a-card>
            <a-statistic title="语言支持" :value="overview.totalLanguages">
              <template #prefix>
                <LanguageOutline style="color: #f0a020" />
              </template>
            </a-statistic>
          </a-card>
        </a-col>
        <a-col :span="6">
          <a-card>
            <a-statistic title="文件总数" :value="overview.totalFiles">
              <template #prefix>
                <FolderOutline style="color: #d03050" />
              </template>
            </a-statistic>
          </a-card>
        </a-col>
      </a-row>

      <!-- 图表区域 -->
      <div class="charts-section">
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
                <a-table
                  :columns="categoryColumns"
                  :data-source="categoryDistribution"
                  :pagination="{ pageSize: 5 }"
                  size="small"
                  row-key="categoryName"
                />
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
                <a-table
                  :columns="languageColumns"
                  :data-source="languagePopularity"
                  :pagination="{ pageSize: 5 }"
                  size="small"
                  row-key="languageName"
                />
              </div>
            </a-col>
          </a-row>
        </a-card>

        <!-- 复杂度和趋势分析 -->
        <a-card title="模板复杂度和使用趋势">
          <a-row :gutter="20">
            <a-col :span="8">
              <div class="chart-container">
                <h3>模板复杂度分析</h3>
                <v-chart class="chart" :option="complexityChartOption" />
              </div>
            </a-col>
            <a-col :span="8">
              <div class="chart-container">
                <h3>创建趋势</h3>
                <v-chart class="chart" :option="trendsChartOption" />
              </div>
            </a-col>
            <a-col :span="8">
              <div class="trends-control">
                <h3>趋势范围</h3>
                <a-select
                  v-model:value="trendsRange"
                  :options="trendsRangeOptions"
                  @change="onTrendsRangeChange"
                  class="trends-select"
                />
              </div>
            </a-col>
          </a-row>
        </a-card>
      </div>
    </a-spin>
  </div>
</template>

<script setup>
  import { ref, onMounted, computed, h } from 'vue';
  import { useRouter } from 'vue-router';
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
  import {
    DocumentTextOutline,
    LayersOutline,
    LanguageOutline,
    FolderOutline,
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

  const router = useRouter();

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

  // 数据状态
  const loading = ref(true);
  const overview = ref({
    totalTemplates: 0,
    totalCategories: 0,
    totalLanguages: 0,
    totalFiles: 0,
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

  // 图表配置
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
    } finally {
      loading.value = false;
    }
  }

  // 生命周期
  onMounted(() => {
    loadStatistics();
  });

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
</script>

<style scoped>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .charts-section {
    margin-top: 24px;
  }

  .chart-container h3,
  .category-table h3,
  .language-table h3,
  .trends-control h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
    color: #333;
  }

  .chart {
    height: 300px;
    width: 100%;
  }

  .trends-control {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100%;
  }

  .trends-select {
    width: 120px;
  }
</style>
