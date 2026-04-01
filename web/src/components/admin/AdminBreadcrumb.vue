<template>
  <div class="breadcrumb-container">
    <n-breadcrumb class="admin-breadcrumb">
      <n-breadcrumb-item
        v-for="(item, index) in breadcrumbItems"
        :key="`breadcrumb-${index}-${item.title}`"
        :clickable="item.clickable"
        @click="item.clickable ? handleBreadcrumbClick(item) : null"
      >
        <template #icon v-if="item.icon">
          <n-icon>
            <component :is="item.icon" />
          </n-icon>
        </template>
        {{ item.title }}
      </n-breadcrumb-item>
    </n-breadcrumb>
  </div>
</template>

<script setup>
  import { computed } from 'vue';
  import { useRoute, useRouter } from 'vue-router';
  import { NBreadcrumb, NBreadcrumbItem, NIcon } from 'naive-ui';
  import {
    GridOutline,
    DocumentTextOutline,
    LayersOutline,
    LanguageOutline,
    StatsChartOutline,
    AddOutline,
    CreateOutline,
    PeopleOutline,
    LockClosedOutline,
    PricetagOutline,
    OptionsOutline,
  } from '@vicons/ionicons5';

  const route = useRoute();
  const router = useRouter();

  // 路由对应的面包屑配置
  const breadcrumbConfig = {
    'admin-dashboard': {
      title: '首页',
      icon: GridOutline,
      path: '/admin',
    },
    'admin-templates-list': {
      title: '模板管理',
      icon: DocumentTextOutline,
      path: '/admin/templates',
    },
    'admin-templates-create': {
      title: '创建模板',
      icon: AddOutline,
      parent: 'admin-templates-list',
      path: '/admin/templates/create',
    },
    'admin-templates-edit': {
      title: '编辑模板',
      icon: CreateOutline,
      parent: 'admin-templates-list',
      path: '/admin/templates/edit',
    },
    // 基础数据分组
    'admin-categories': {
      title: '分类管理',
      icon: LayersOutline,
      parent: 'basic-data',
      path: '/admin/categories',
    },
    'admin-languages': {
      title: '语言管理',
      icon: LanguageOutline,
      parent: 'basic-data',
      path: '/admin/languages',
    },
    'admin-tags': {
      title: '标签管理',
      icon: PricetagOutline,
      parent: 'basic-data',
      path: '/admin/tags',
    },
    'admin-var-presets': {
      title: '变量预设',
      icon: OptionsOutline,
      parent: 'basic-data',
      path: '/admin/var-presets',
    },
    // 用户管理分组
    'admin-users': {
      title: '用户管理',
      icon: PeopleOutline,
      parent: 'user-management',
      path: '/admin/users',
    },
    'admin-permissions': {
      title: '权限管理',
      icon: LockClosedOutline,
      parent: 'user-management',
      path: '/admin/permissions',
    },
  };

  // 虚拟父级配置（对应菜单分组）
  const virtualParents = {
    'basic-data': {
      title: '基础数据',
      icon: null,
      path: null,
    },
    'user-management': {
      title: '用户管理',
      icon: null,
      path: null,
    },
  };

  const breadcrumbItems = computed(() => {
    const currentRouteName = route.name;
    const items = [];

    if (currentRouteName && breadcrumbConfig[currentRouteName]) {
      const currentConfig = breadcrumbConfig[currentRouteName];

      // 如果有父级，添加父级
      if (currentConfig.parent) {
        // 检查是否是虚拟父级（菜单分组）
        if (virtualParents[currentConfig.parent]) {
          const virtualParent = virtualParents[currentConfig.parent];
          items.push({
            title: virtualParent.title,
            icon: virtualParent.icon,
            path: virtualParent.path,
            clickable: false, // 虚拟父级不可点击
          });
        } else {
          // 真实的父级页面
          const parentConfig = breadcrumbConfig[currentConfig.parent];
          if (parentConfig) {
            items.push({
              title: parentConfig.title,
              icon: parentConfig.icon,
              path: parentConfig.path,
              clickable: true,
            });
          }
        }
      }

      // 添加当前页面
      items.push({
        title: currentConfig.title,
        icon: currentConfig.icon,
        path: currentConfig.path,
        clickable: false,
      });
    } else {
      // 默认显示仪表盘
      items.push({
        title: '仪表盘',
        icon: GridOutline,
        path: '/admin',
        clickable: false,
      });
    }

    return items;
  });

  function handleBreadcrumbClick(item) {
    if (item.clickable && item.path) {
      router.push(item.path);
    }
  }
</script>

<style scoped>
  .breadcrumb-container {
    padding: 16px 24px;
    background: #fff;
    border-bottom: 1px solid #f0f0f0;
  }

  .admin-breadcrumb {
    font-size: 14px;
  }

  :deep(.n-breadcrumb-item:not(:last-child) .n-breadcrumb-item__link) {
    color: #666;
    transition: color 0.2s ease;
  }

  :deep(.n-breadcrumb-item:not(:last-child) .n-breadcrumb-item__link:hover) {
    color: #18a058;
  }

  :deep(.n-breadcrumb-item:last-child .n-breadcrumb-item__link) {
    color: #333;
    font-weight: 500;
  }

  :deep(.n-breadcrumb-item__icon) {
    margin-right: 4px;
  }
</style>
