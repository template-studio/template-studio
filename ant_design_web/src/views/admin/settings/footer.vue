<template>
  <a-spin :spinning="loading">
    <a-form layout="vertical">
      <!-- 链接管理 -->
      <a-form-item label="页脚链接">
        <div class="links-editor">
          <div v-for="(link, index) in footerLinks" :key="index" class="link-item">
            <a-input v-model:value="link.label" placeholder="链接文本" style="width: 200px" />
            <a-input v-model:value="link.url" placeholder="链接地址" style="flex: 1" />
            <a-button danger @click="removeLink(index)">
              <template #icon><CloseOutline /></template>
            </a-button>
          </div>
          <a-button type="dashed" block @click="addLink">
            <template #icon><AddOutline /></template>
            添加链接
          </a-button>
        </div>
      </a-form-item>

      <!-- 版权信息 -->
      <a-form-item label="版权信息">
        <a-input v-model:value="copyright" placeholder="© 2025 Your Company" />
      </a-form-item>

      <!-- 技术栈信息 -->
      <a-form-item label="技术栈信息">
        <a-input v-model:value="poweredBy" placeholder="基于 Rust & Vue3 构建" />
      </a-form-item>

      <!-- 联系方式 -->
      <a-form-item label="反馈邮箱">
        <a-input v-model:value="feedbackEmail" placeholder="feedback@example.com" />
      </a-form-item>
      <a-form-item label="技术支持邮箱">
        <a-input v-model:value="supportEmail" placeholder="support@example.com" />
      </a-form-item>

      <!-- 保存按钮 -->
      <a-form-item>
        <a-button type="primary" :loading="saving" @click="handleSave">
          保存设置
        </a-button>
      </a-form-item>
    </a-form>
  </a-spin>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { AddOutline, CloseOutline } from '@/icons/ionicons5';
import { message } from 'ant-design-vue';
import { getSettings, batchUpdateSettings } from '@/api/system/settings';

interface FooterLink {
  label: string;
  url: string;
}

const loading = ref(false);
const saving = ref(false);
const footerLinks = ref<FooterLink[]>([]);
const copyright = ref('');
const poweredBy = ref('');
const feedbackEmail = ref('');
const supportEmail = ref('');

function addLink() {
  footerLinks.value.push({ label: '', url: '#' });
}

function removeLink(index: number) {
  footerLinks.value.splice(index, 1);
}

async function loadSettings() {
  loading.value = true;
  try {
    const res = await getSettings({ group: 'footer' });
    const items = res.data?.data || [];
    for (const item of items) {
      switch (item.key) {
        case 'links':
          try {
            footerLinks.value = JSON.parse(item.value || '[]');
          } catch {
            footerLinks.value = [];
          }
          break;
        case 'copyright':
          copyright.value = item.value || '';
          break;
        case 'powered_by':
          poweredBy.value = item.value || '';
          break;
        case 'contact':
          try {
            const contact = JSON.parse(item.value || '{}');
            feedbackEmail.value = contact.feedback_email || '';
            supportEmail.value = contact.support_email || '';
          } catch {
            feedbackEmail.value = '';
            supportEmail.value = '';
          }
          break;
      }
    }
  } catch (e) {
    message.error('加载设置失败');
  } finally {
    loading.value = false;
  }
}

async function handleSave() {
  saving.value = true;
  try {
    await batchUpdateSettings({
      group: 'footer',
      items: [
        { key: 'links', value: JSON.stringify(footerLinks.value) },
        { key: 'copyright', value: copyright.value },
        { key: 'powered_by', value: poweredBy.value },
        {
          key: 'contact',
          value: JSON.stringify({
            feedback_email: feedbackEmail.value,
            support_email: supportEmail.value,
          }),
        },
      ],
    });
    message.success('保存成功');
  } catch (e) {
    message.error('保存失败');
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  loadSettings();
});
</script>

<style scoped>
.links-editor {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.link-item {
  display: flex;
  gap: 8px;
  align-items: center;
}
</style>
