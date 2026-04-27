<template>
  <div class="logo" :class="{ 'logo-dark': dark }">
    <img :src="websiteConfig.logo" alt="" :class="{ 'mr-2': !collapsed }" />
    <h2 v-show="!collapsed" class="title">{{ websiteConfig.title }}</h2>
  </div>
</template>

<script lang="ts">
  import { websiteConfig } from '@/config/website.config';
  export default {
    name: 'Index',
    props: {
      collapsed: {
        type: Boolean,
      },
      dark: {
        type: Boolean,
        default: false,
      },
    },
    data() {
      return {
        websiteConfig,
      };
    },
  };
</script>

<style lang="less" scoped>
  .logo {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 64px;
    line-height: 64px;
    overflow: hidden;
    white-space: nowrap;

    img {
      width: auto;
      height: 32px;
    }

    .title {
      margin: 0;
      color: #515a6e;
    }

    &-dark {
      .title {
        color: #fff;
      }
    }
  }
</style>
