<template>
  <footer class="footer-bar">
    <div class="footer-main">
      <div class="footer-links">
        <a
          v-for="link in footerLinks"
          :key="link.label"
          :href="link.url"
          @click.prevent="link.url !== '#' && ($event.target.removeAttribute('href'))"
        >{{ link.label }}</a>
      </div>
      <div class="footer-info">
        <span>{{ copyright }}</span>
        <span v-if="poweredBy" class="footer-divider">|</span>
        <span v-if="poweredBy">{{ poweredBy }}</span>
      </div>
      <div v-if="feedbackEmail || supportEmail" class="footer-contact">
        <span v-if="feedbackEmail">反馈邮箱：{{ feedbackEmail }}</span>
        <span v-if="feedbackEmail && supportEmail" class="footer-divider">|</span>
        <span v-if="supportEmail">技术支持：{{ supportEmail }}</span>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getPublicSettings } from '@/api/system/settings';

interface FooterLink {
  label: string;
  url: string;
}

const footerLinks = ref<FooterLink[]>([
  { label: '关于我们', url: '#' },
  { label: '用户协议', url: '#' },
  { label: '隐私政策', url: '#' },
  { label: '联系我们', url: '#' },
  { label: '友情链接', url: '#' },
]);
const copyright = ref('© 2025 Template Studio');
const poweredBy = ref('基于 Rust & Vue3 构建 | Powered by Naive UI & Vite');
const feedbackEmail = ref('feedback@templateStudio.com');
const supportEmail = ref('support@templateStudio.com');

onMounted(async () => {
  try {
    const res = await getPublicSettings('footer');
    const items = res.data?.data || [];
    for (const item of items) {
      switch (item.key) {
        case 'links':
          try {
            const parsed = JSON.parse(item.value || '[]');
            if (parsed.length > 0) footerLinks.value = parsed;
          } catch { /* keep defaults */ }
          break;
        case 'copyright':
          if (item.value) copyright.value = item.value;
          break;
        case 'powered_by':
          if (item.value) poweredBy.value = item.value;
          break;
        case 'contact':
          try {
            const contact = JSON.parse(item.value || '{}');
            if (contact.feedback_email) feedbackEmail.value = contact.feedback_email;
            if (contact.support_email) supportEmail.value = contact.support_email;
          } catch { /* keep defaults */ }
          break;
      }
    }
  } catch {
    // API 不可用时使用默认值
  }
});
</script>

<style scoped>
  .footer-bar {
    position: static;
    width: 100%;
    background: #f8f9fa;
    border-top: 1px solid #ececec;
    z-index: 100;
    padding: 32px 0 18px 0;
    margin-top: 48px;
  }

  .footer-main {
    width: 80vw;
    max-width: 1280px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .footer-links {
    display: flex;
    gap: 24px;
    font-size: 15px;
    color: #666;
    margin-bottom: 4px;
  }

  .footer-links a {
    color: #666;
    text-decoration: none;
    transition: color 0.2s;
  }

  .footer-links a:hover {
    color: #18a058;
  }

  .footer-info {
    color: #888;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .footer-divider {
    margin: 0 6px;
    color: #ccc;
  }

  .footer-contact {
    color: #aaa;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
