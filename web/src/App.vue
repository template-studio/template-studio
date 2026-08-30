<template>
  <a-config-provider
    :locale="zhCN"
    :theme="themeConfig"
  >
    <AppProvider>
      <RouterView />
    </AppProvider>
  </a-config-provider>
</template>

<script lang="ts" setup>
  import { computed, reactive, watch } from 'vue';
  import zhCN from 'ant-design-vue/locale/zh_CN';
  import { theme } from 'ant-design-vue';
  import { AppProvider } from '@/components/Application';
  import { useDesignSettingStore } from '@/store/modules/designSetting';

  const designStore = useDesignSettingStore();

  const themeConfig = reactive({
    algorithm: theme.defaultAlgorithm,
    token: {
      colorPrimary: '#2d8cf0',
    },
  });

  watch(
    () => [designStore.appTheme, designStore.darkTheme],
    ([appTheme, isDark]) => {
      themeConfig.algorithm = isDark ? theme.darkAlgorithm : theme.defaultAlgorithm;
      themeConfig.token.colorPrimary = appTheme as string;
    },
    { immediate: true }
  );
</script>
