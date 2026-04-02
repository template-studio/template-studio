<template>
  <div class="template-review-page">
    <n-card :bordered="false">
      <n-flex vertical>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
          <h2 style="margin: 0; font-size: 18px">待审核模板</h2>
          <n-button @click="loadData">
            <template #icon><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
            刷新
          </n-button>
        </div>

        <n-data-table
          :columns="columns"
          :data="templates"
          :loading="loading"
          :pagination="pagination"
          @update:page="handlePageChange"
        />
      </n-flex>
    </n-card>

    <!-- 审核弹窗 -->
    <n-modal v-model:show="showReviewModal" :mask-closable="false">
      <n-card style="width: 450px" title="审核模板" :bordered="false" size="huge" role="dialog">
        <template #header-extra>
          <n-button quaternary circle @click="showReviewModal = false">
            <template #icon><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></template>
          </n-button>
        </template>
        <div v-if="reviewingTemplate" style="margin-bottom: 16px">
          <p style="font-weight: 600; margin: 0 0 4px">{{ reviewingTemplate.name }}</p>
          <p style="font-size: 13px; color: #94a3b8; margin: 0">{{ reviewingTemplate.description }}</p>
        </div>
        <n-form label-placement="top">
          <n-form-item label="审核备注">
            <n-input v-model:value="reviewReason" type="textarea" placeholder="填写审核意见（拒绝时必填）" :rows="3" />
          </n-form-item>
        </n-form>
        <template #footer>
          <div style="display: flex; gap: 12px; justify-content: flex-end">
            <n-button @click="showReviewModal = false">取消</n-button>
            <n-button type="error" @click="handleReview('reject')" :loading="reviewing">拒绝</n-button>
            <n-button type="primary" @click="handleReview('approve')" :loading="reviewing">通过</n-button>
          </div>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, h, onMounted } from 'vue';
import { useMessage, NButton, NTag, NSpace } from 'naive-ui';
import { listPendingTemplates, reviewTemplate } from '@/api/templates/contribution';

const message = useMessage();
const templates = ref([]);
const loading = ref(false);
const currentPage = ref(1);
const pageSize = 20;
const total = ref(0);

const showReviewModal = ref(false);
const reviewingTemplate = ref(null);
const reviewReason = ref('');
const reviewing = ref(false);

const pagination = {
  page: 1,
  pageSize,
  itemCount: 0,
  onChange: (page) => { currentPage.value = page; loadData(); },
};

const columns = [
  { title: 'ID', key: 'id', width: 160, ellipsis: { tooltip: true } },
  { title: '模板名称', key: 'name', ellipsis: { tooltip: true } },
  { title: '描述', key: 'description', ellipsis: { tooltip: true } },
  { title: '类型', key: 'templateType', width: 100, render: (row) => {
    const map = { basic: '基础', scaffold: '脚手架', data_driven: '数据驱动' };
    return map[row.templateType] || row.templateType;
  }},
  { title: '创建时间', key: 'createdAt', width: 200 },
  { title: '操作', key: 'actions', width: 100, render: (row) => {
    return h(NSpace, { size: 'small' }, () => [
      h(NButton, { size: 'small', type: 'primary', onClick: () => openReview(row) }, () => '审核'),
    ]);
  }},
];

async function loadData() {
  loading.value = true;
  try {
    const res = await listPendingTemplates({ page: currentPage.value, pageSize });
    templates.value = res?.templatesList || [];
    total.value = res?.total || 0;
    pagination.itemCount = total.value;
    pagination.page = currentPage.value;
  } catch {
    message.error('加载失败');
  } finally {
    loading.value = false;
  }
}

function handlePageChange(page) {
  currentPage.value = page;
  loadData();
}

function openReview(tmpl) {
  reviewingTemplate.value = tmpl;
  reviewReason.value = '';
  showReviewModal.value = true;
}

async function handleReview(action) {
  if (action === 'reject' && !reviewReason.value.trim()) {
    message.warning('拒绝时请填写原因');
    return;
  }
  reviewing.value = true;
  try {
    await reviewTemplate({
      templateId: reviewingTemplate.value.id,
      action,
      reason: reviewReason.value,
    });
    message.success(action === 'approve' ? '已通过' : '已拒绝');
    showReviewModal.value = false;
    loadData();
  } catch {
    message.error('审核失败');
  } finally {
    reviewing.value = false;
  }
}

onMounted(() => loadData());
</script>

<style scoped>
.template-review-page {
  padding: 0;
}
</style>
