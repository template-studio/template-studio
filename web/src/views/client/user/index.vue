<template>
  <div class="user-page">
    <!-- Profile Header -->
    <div class="profile-header">
      <div class="header-bg">
        <div class="header-glow"></div>
      </div>
      <div class="header-content">
        <div class="user-avatar">
          <img v-if="userInfo.avatar" :src="avatarFullUrl" alt="avatar" class="avatar-img" />
          <div v-else class="avatar-placeholder">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
          </div>
        </div>
        <div class="user-meta">
          <h1 class="user-name">{{ userInfo.username }}</h1>
          <p v-if="userInfo.bio" class="user-bio">{{ userInfo.bio }}</p>
          <div class="user-stats">
            <div class="stat">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
              <span>{{ templates.length }} 个模板</span>
            </div>
            <div class="stat">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/></svg>
              <span>加入于 {{ userInfo.createdAt }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Templates Section -->
    <div class="templates-section">
      <div class="section-container">
        <div v-if="loading" style="text-align: center; padding: 60px 0; color: #94a3b8">加载中...</div>
        <div v-else-if="error" style="text-align: center; padding: 60px 0; color: #ef4444">{{ error }}</div>
        <template v-else>
          <div v-if="templates.length === 0" class="empty-state">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#cbd5e1" stroke-width="1.5"><path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"/></svg>
            <p>该用户暂无公开模板</p>
          </div>
          <div v-else class="templates-grid">
            <TemplateCard
              v-for="template in templates"
              :key="template.id"
              :template="template"
              @click="goTemplate"
            />
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { getPublicProfile } from '@/api/system/user';
import TemplateCard from '@/components/TemplateCard.vue';

const route = useRoute();
const router = useRouter();

const loading = ref(true);
const error = ref('');
const userInfo = ref({ username: '', avatar: '', bio: '', createdAt: '' });
const templates = ref([]);

const avatarFullUrl = computed(() => {
  const avatar = userInfo.value.avatar;
  if (!avatar) return '';
  if (avatar.startsWith('http')) return avatar;
  return `http://localhost:8001${avatar}`;
});

onMounted(async () => {
  const username = route.params.username;
  if (!username) {
    error.value = '用户不存在';
    loading.value = false;
    return;
  }
  try {
    const res = await getPublicProfile(username);
    if (res.data?.code === 0) {
      userInfo.value = res.data.data;
      templates.value = res.data.data.templates || [];
    } else {
      error.value = res.data?.message || '加载失败';
    }
  } catch (e) {
    error.value = e?.response?.data?.message || '用户不存在';
  } finally {
    loading.value = false;
  }
});

function goTemplate(template) {
  router.push(`/template-generator/${template.id}`);
}
</script>

<style scoped>
.user-page {
  min-height: calc(100vh - 64px);
  background: #f8fafc;
}

/* ===== Header ===== */
.profile-header {
  position: relative;
  background: #0f172a;
  overflow: hidden;
}

.header-bg {
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%);
}

.header-glow {
  position: absolute;
  width: 400px;
  height: 400px;
  top: -150px;
  right: -100px;
  border-radius: 50%;
  background: rgba(34, 197, 94, 0.08);
  filter: blur(80px);
}

.header-content {
  position: relative;
  z-index: 1;
  max-width: 1200px;
  margin: 0 auto;
  padding: 48px 40px 40px;
  display: flex;
  align-items: center;
  gap: 28px;
}

.user-avatar {
  width: 96px;
  height: 96px;
  border-radius: 50%;
  overflow: hidden;
  background: #1e293b;
  border: 3px solid rgba(255, 255, 255, 0.1);
  flex-shrink: 0;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #334155;
}

.user-meta {
  color: #fff;
}

.user-name {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px;
  letter-spacing: -0.5px;
}

.user-bio {
  font-size: 15px;
  color: #94a3b8;
  margin: 0 0 14px;
  line-height: 1.6;
  max-width: 500px;
}

.user-stats {
  display: flex;
  gap: 20px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: #64748b;
}

/* ===== Templates ===== */
.templates-section {
  padding: 40px 0 60px;
}

.section-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 40px;
}

.templates-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 80px 20px;
  color: #94a3b8;
}

.empty-state p {
  font-size: 15px;
  margin-top: 16px;
  color: #64748b;
}

@media (max-width: 768px) {
  .header-content {
    flex-direction: column;
    text-align: center;
    padding: 40px 20px 32px;
  }

  .user-stats {
    justify-content: center;
  }

  .user-bio {
    max-width: 100%;
  }

  .section-container {
    padding: 0 20px;
  }

  .templates-grid {
    grid-template-columns: 1fr;
  }
}
</style>
