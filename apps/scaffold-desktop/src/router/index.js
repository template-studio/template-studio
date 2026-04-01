import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: HomeView
  },
  {
    path: '/templates',
    name: 'Templates',
    component: () => import('@/views/TemplatesView.vue')
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/SettingsView.vue')
  },
  // 添加所有深层链接的通配符路由
  {
    path: '/settings/:mainTab?',
    name: 'SettingsWithMainTab',
    component: () => import('@/views/SettingsView.vue')
  },
  {
    path: '/settings/:mainTab/:subTab?',
    name: 'SettingsWithSubTab',
    component: () => import('@/views/SettingsView.vue')
  },
  {
    path: '/settings/:mainTab/:subTab/:thirdTab?',
    name: 'SettingsWithThirdTab',
    component: () => import('@/views/SettingsView.vue')
  },
  {
    path: '/help',
    name: 'Help',
    component: () => import('@/views/HelpView.vue')
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

// Navigation guard for development
if (import.meta.env.DEV) {
  router.beforeEach((to, from, next) => {
    console.log(`Navigating from ${from.path} to ${to.path}`)
    next()
  })
}

export default router