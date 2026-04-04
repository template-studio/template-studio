import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'

const routes = [
  {
    path: '/',
    redirect: '/home'
  },
  {
    path: '/home',
    name: 'Home',
    component: HomeView
  },
  {
    path: '/templates',
    name: 'Templates',
    component: () => import('@/views/TemplatesView.vue')
  },
  {
    path: '/datasource',
    name: 'DataSource',
    component: () => import('@/views/DataSourceView.vue')
  },
  {
    path: '/projects',
    name: 'Projects',
    component: () => import('@/views/ProjectsView.vue')
  },
  {
    path: '/mappings',
    name: 'Mappings',
    component: () => import('@/views/MappingsView.vue')
  },
  {
    path: '/languages',
    name: 'Languages',
    component: () => import('@/views/LanguagesView.vue')
  },
  {
    path: '/project/:id',
    name: 'ProjectWorkspace',
    redirect: to => `/project/${to.params.id}/tables`,
    component: () => import('@/views/ProjectWorkspaceView.vue')
  },
  {
    path: '/project/:id/tables',
    name: 'ProjectTables',
    component: () => import('@/views/project/TablesView.vue')
  },
  {
    path: '/project/:id/preferences',
    name: 'ProjectPreferences',
    component: () => import('@/views/project/PreferencesView.vue')
  },
  {
    path: '/project/:id/mappings',
    name: 'ProjectMappings',
    component: () => import('@/views/project/MappingsView.vue')
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