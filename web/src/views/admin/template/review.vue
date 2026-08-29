<template>
  <div class="template-review-page">
    <a-card :bordered="false">
      <div style="display: flex; flex-direction: column; gap: 16px">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
          <h2 style="margin: 0; font-size: 18px">待审核模板</h2>
          <a-button @click="loadData">
            <template #icon><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg></template>
            刷新
          </a-button>
        </div>

        <a-table
          :columns="columns"
          :data-source="templates"
          :loading="loading"
          :pagination="pagination"
          row-key="id"
        />
      </div>
    </a-card>

    <!-- 审核弹窗 -->
    <a-modal v-model:open="showReviewModal" :mask-closable="false" title="审核模板" width="450px">
      <div v-if="reviewingTemplate" style="margin-bottom: 16px">
        <p style="font-weight: 600; margin: 0 0 4px">{{ reviewingTemplate.name }}</p>
        <p style="font-size: 13px; color: #94a3b8; margin: 0">{{ reviewingTemplate.description }}</p>
      </div>
      <a-form layout="vertical">
        <a-form-item label="审核备注">
          <a-textarea v-model:value="reviewReason" placeholder="填写审核意见（拒绝时必填）" :rows="3" />
        </a-form-item>
      </a-form>
      <template #footer>
        <div style="display: flex; gap: 12px; justify-content: flex-end">
          <a-button @click="showReviewModal = false">取消</a-button>
          <a-button danger @click="handleReview('reject')" :loading="reviewing">拒绝</a-button>
          <a-button type="primary" @click="handleReview('approve')" :loading="reviewing">通过</a-button>
        </div>
      </template>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, h, onMounted } from 'vue';
import { Button, Space, message } from 'ant-design-vue';
import { listPendingTemplates, reviewTemplate } from '@/api/templates/contribution';

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
  current: currentPage,
  pageSize,
  total,
  onChange: (page) => { currentPage.value = page; loadData(); },
};

const columns = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 160, ellipsis: true },
  { title: '模板名称', dataIndex: 'name', key: 'name', ellipsis: true },
  { title: '描述', dataIndex: 'description', key: 'description', ellipsis: true },
  { title: '类型', dataIndex: 'templateType', key: 'templateType', width: 100, customRender: ({ text }) => {
    const map = { basic: '基础', scaffold: '脚手架', data_driven: '数据驱动' };
    return map[text] || text;
  }},
  { title: '创建时间', dataIndex: 'createdAt', key: 'createdAt', width: 200 },
  { title: '操作', dataIndex: 'actions', key: 'actions', width: 100, customRender: ({ record }) => {
    return h(Space, { size: 'small' }, () => [
      h(Button, { size: 'small', type: 'primary', onClick: () => openReview(record) }, () => '审核'),
    ]);
  }},
];

async function loadData() {
  loading.value = true;
  try {
    const res = await listPendingTemplates({ page: currentPage.value, pageSize });
    templates.value = res?.templatesList || [];
    total.value = res?.total || 0;
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
