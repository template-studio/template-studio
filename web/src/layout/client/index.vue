<template>
  <NavBar />
  <div class="main-content">
    <router-view v-slot="{ Component }">
      <transition mode="out-in" @before-enter="beforeEnter" @enter="enter" @leave="leave">
        <component :is="Component" />
      </transition>
    </router-view>
  </div>
  <FooterBar />
</template>

<script setup>
  import NavBar from '@/components/NavBar.vue';
  import FooterBar from '@/components/FooterBar.vue';
  import Container from '@/components/Container.vue';

  function beforeEnter(el) {
    el.style.opacity = 0;
  }

  function enter(el, done) {
    el.offsetHeight; // trigger reflow
    el.style.transition = 'opacity 0.3s ease';
    el.style.opacity = 1;
    setTimeout(done, 300);
  }

  function leave(el, done) {
    el.style.transition = 'opacity 0.2s ease';
    el.style.opacity = 0;
    setTimeout(done, 200);
  }
</script>

<style scoped>
  .main-content {
    background: #f5f5f5;
    width: 100%;
    padding-top: 64px;
    padding-bottom: 20px;
    overflow-x: hidden;
  }

  /* 防止 transition 动画期间出现滚动条 */
  :deep(.v-enter-active),
  :deep(.v-leave-active) {
    transition: opacity 0.3s ease;
  }

  :deep(.v-enter-from),
  :deep(.v-leave-to) {
    opacity: 0;
  }
</style>
