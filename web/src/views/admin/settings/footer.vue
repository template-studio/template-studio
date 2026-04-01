<template>
  <div class="settings-footer">
    <n-card title="Footer 设置" :bordered="false">
      <n-spin :show="loading">
        <n-form label-placement="top">
          <!-- 链接管理 -->
          <n-form-item label="页脚链接">
            <div class="links-editor">
              <div v-for="(link, index) in footerLinks" :key="index" class="link-item">
                <n-input v-model:value="link.label" placeholder="链接文本" style="width: 200px" />
                <n-input v-model:value="link.url" placeholder="链接地址" style="flex: 1" />
                <n-button quaternary type="error" @click="removeLink(index)">
                  <template #icon><n-icon><CloseOutline /></n-icon></template>
                </n-button>
              </div>
              <n-button dashed block @click="addLink">
                <template #icon><n-icon><AddOutline /></n-icon></template>
                添加链接
              </n-button>
            </div>
          </n-form-item>

          <!-- 版权信息 -->
          <n-form-item label="版权信息">
            <n-input v-model:value="copyright" placeholder="© 2025 Your Company" />
          </n-form-item>

          <!-- 技术栈信息 -->
          <n-form-item label="技术栈信息">
            <n-input v-model:value="poweredBy" placeholder="基于 Rust & Vue3 构建" />
          </n-form-item>

          <!-- 联系方式 -->
          <n-form-item label="反馈邮箱">
            <n-input v-model:value="feedbackEmail" placeholder="feedback@example.com" />
          </n-form-item>
          <n-form-item label="技术支持邮箱">
            <n-input v-model:value="supportEmail" placeholder="support@example.com" />
          </n-form-item>

          <!-- 保存按钮 -->
          <n-form-item>
            <n-button type="primary" :loading="saving" @click="handleSave">
              保存设置
            </n-button>
          </n-form-item>
        </n-form>
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useMessage } from 'naive-ui';
import { AddOutline, CloseOutline } from '@vicons/ionicons5';
import { getSettings, batchUpdateSettings } from '@/api/system/settings';

const message = useMessage();

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
.settings-footer {
  padding: 16px;
}

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
