<template>
  <a-config-provider
    v-if="!isLock"
    :locale="zhCN"
    :theme="themeConfig"
  >
    <AppProvider>
      <RouterView />
    </AppProvider>
  </a-config-provider>

  <transition v-if="isLock && $route.name !== 'login'" name="slide-up">
    <LockScreen />
  </transition>
</template>

<script lang="ts" setup>
  import { computed, reactive, watch, onMounted, onUnmounted } from 'vue';
  import zhCN from 'ant-design-vue/locale/zh_CN';
  import { theme } from 'ant-design-vue';
  import { LockScreen } from '@/components/Lockscreen';
  import { AppProvider } from '@/components/Application';
  import { useScreenLockStore } from '@/store/modules/screenLock';
  import { useRoute } from 'vue-router';
  import { useDesignSettingStore } from '@/store/modules/designSetting';

  const route = useRoute();
  const useScreenLock = useScreenLockStore();
  const designStore = useDesignSettingStore();
  const isLock = computed(() => useScreenLock.isLocked);
  const lockTime = computed(() => useScreenLock.lockTime);

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
      themeConfig.token.colorPrimary = appTheme;
    },
    { immediate: true }
  );

  let timer: NodeJS.Timer;

  const timekeeping = () => {
    clearInterval(timer);
    if (route.name == 'login' || isLock.value) return;
    // 设置不锁屏
    useScreenLock.setLock(false);
    // 重置锁屏时间
    useScreenLock.setLockTime();
    timer = setInterval(() => {
      // 锁屏倒计时递减
      useScreenLock.setLockTime(lockTime.value - 1);
      if (lockTime.value <= 0) {
        // 设置锁屏
        useScreenLock.setLock(true);
        return clearInterval(timer);
      }
    }, 1000);
  };

  onMounted(() => {
    document.addEventListener('mousedown', timekeeping);
  });

  onUnmounted(() => {
    document.removeEventListener('mousedown', timekeeping);
  });
</script>
