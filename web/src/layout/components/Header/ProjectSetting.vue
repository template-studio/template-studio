<template>
  <a-drawer v-model:open="isDrawer" :width="width" :placement="placement" :title="title">
    <div class="drawer">
      <a-divider>主题</a-divider>

      <div class="justify-center drawer-setting-item dark-switch">
        <a-tooltip placement="bottom">
          <template #title>{{ designStore.darkTheme ? '深' : '浅' }}色主题</template>
          <a-switch v-model:checked="designStore.darkTheme" class="dark-theme-switch">
            <template #checkedChildren>
              <span style="font-size: 14px">🌙</span>
            </template>
            <template #unCheckedChildren>
              <span style="font-size: 14px">☀️</span>
            </template>
          </a-switch>
        </a-tooltip>
      </div>

      <a-divider>系统主题</a-divider>

      <div class="drawer-setting-item align-items-top">
        <span
          class="theme-item"
          v-for="(item, index) in appThemeList"
          :key="index"
          :style="{ 'background-color': item }"
          @click="togTheme(item)"
        >
          <CheckOutlined v-if="item === designStore.appTheme" class="theme-check-icon" />
        </span>
      </div>

      <a-divider>导航栏模式</a-divider>

      <div class="drawer-setting-item align-items-top">
        <div class="drawer-setting-item-style align-items-top">
          <a-tooltip placement="top">
            <template #title>左侧菜单模式</template>
            <img
              src="~@/assets/images/nav-theme-dark.svg"
              @click="togNavMode('vertical')"
              alt="左侧菜单模式"
            />
          </a-tooltip>
          <a-badge dot color="#19be6b" v-show="settingStore.navMode === 'vertical'" />
        </div>

        <div class="drawer-setting-item-style">
          <a-tooltip placement="top">
            <template #title>顶部菜单模式</template>
            <img
              src="~@/assets/images/nav-horizontal.svg"
              alt="顶部菜单模式"
              @click="togNavMode('horizontal')"
            />
          </a-tooltip>
          <a-badge dot color="#19be6b" v-show="settingStore.navMode === 'horizontal'" />
        </div>

        <div class="drawer-setting-item-style">
          <a-tooltip placement="top">
            <template #title>顶部菜单混合模式</template>
            <img
              src="~@/assets/images/nav-horizontal-mix.svg"
              @click="togNavMode('horizontal-mix')"
              alt="顶部菜单混合模式"
            />
          </a-tooltip>
          <a-badge dot color="#19be6b" v-show="settingStore.navMode === 'horizontal-mix'" />
        </div>
      </div>

      <a-divider>导航栏风格</a-divider>

      <div class="drawer-setting-item align-items-top">
        <div class="drawer-setting-item-style align-items-top">
          <a-tooltip placement="top">
            <template #title>暗色侧边栏</template>
            <img
              src="~@/assets/images/nav-theme-dark.svg"
              alt="暗色侧边栏"
              @click="togNavTheme('dark')"
            />
          </a-tooltip>
          <a-badge dot color="#19be6b" v-if="settingStore.navTheme === 'dark'" />
        </div>

        <div class="drawer-setting-item-style">
          <a-tooltip placement="top">
            <template #title>白色侧边栏</template>
            <img
              src="~@/assets/images/nav-theme-light.svg"
              alt="白色侧边栏"
              @click="togNavTheme('light')"
            />
          </a-tooltip>
          <a-badge dot color="#19be6b" v-if="settingStore.navTheme === 'light'" />
        </div>

        <div class="drawer-setting-item-style">
          <a-tooltip placement="top">
            <template #title>暗色顶栏</template>
            <img
              src="~@/assets/images/header-theme-dark.svg"
              @click="togNavTheme('header-dark')"
              alt="暗色顶栏"
            />
          </a-tooltip>
          <a-badge dot color="#19be6b" v-if="settingStore.navTheme === 'header-dark'" />
        </div>
      </div>
      <a-divider>界面功能</a-divider>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 分割菜单 </div>
        <div class="drawer-setting-item-action">
          <a-switch
            :disabled="settingStore.navMode !== 'horizontal-mix'"
            v-model:checked="settingStore.menuSetting.mixMenu"
          />
        </div>
      </div>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 固定顶栏 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.headerSetting.fixed" />
        </div>
      </div>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 固定多页签 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.multiTabsSetting.fixed" />
        </div>
      </div>

      <a-divider>界面显示</a-divider>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 显示重载页面按钮 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.headerSetting.isReload" />
        </div>
      </div>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 显示面包屑导航 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.crumbsSetting.show" />
        </div>
      </div>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 显示面包屑显示图标 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.crumbsSetting.showIcon" />
        </div>
      </div>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 显示多页签 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.multiTabsSetting.show" />
        </div>
      </div>

      <a-divider>动画</a-divider>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 启用动画 </div>
        <div class="drawer-setting-item-action">
          <a-switch v-model:checked="settingStore.isPageAnimate" />
        </div>
      </div>

      <div class="drawer-setting-item">
        <div class="drawer-setting-item-title"> 动画类型 </div>
        <div class="drawer-setting-item-select">
          <a-select v-model:value="settingStore.pageAnimateType" :options="animateOptions" />
        </div>
      </div>

      <div class="drawer-setting-item">
        <a-alert type="warning" :show-icon="false">
          <p>{{ alertText }}</p>
        </a-alert>
      </div>
    </div>
  </a-drawer>
</template>

<script lang="ts">
  import { defineComponent, reactive, toRefs, unref, watch, computed } from 'vue';
  import { useProjectSettingStore } from '@/store/modules/projectSetting';
  import { useDesignSettingStore } from '@/store/modules/designSetting';
  import { CheckOutlined } from '@ant-design/icons-vue';
  import { theme } from 'ant-design-vue';
  import { animates as animateOptions } from '@/settings/animateSetting';

  export default defineComponent({
    name: 'ProjectSetting',
    components: { CheckOutlined },
    props: {
      title: {
        type: String,
        default: '项目配置',
      },
      width: {
        type: Number,
        default: 280,
      },
    },
    setup(props) {
      const settingStore = useProjectSettingStore();
      const designStore = useDesignSettingStore();
      const state = reactive({
        width: props.width,
        title: props.title,
        isDrawer: false,
        placement: 'right',
        alertText: '该功能主要实时预览各种布局效果，更多完整配置在 projectSetting.ts 中设置',
        appThemeList: designStore.appThemeList,
      });

      watch(
        () => designStore.darkTheme,
        (to) => {
          settingStore.navTheme = to ? 'header-dark' : 'dark';
        }
      );

      const directionsOptions = computed(() => {
        return animateOptions.find((item) => item.value == unref(settingStore.pageAnimateType));
      });

      function openDrawer() {
        state.isDrawer = true;
      }

      function closeDrawer() {
        state.isDrawer = false;
      }

      function togNavTheme(theme) {
        settingStore.navTheme = theme;
        if (settingStore.navMode === 'horizontal' && ['light'].includes(theme)) {
          settingStore.navTheme = 'dark';
        }
      }

      function togTheme(color) {
        designStore.appTheme = color;
      }

      function togNavMode(mode) {
        settingStore.navMode = mode;
        settingStore.menuSetting.mixMenu = false;
      }

      return {
        ...toRefs(state),
        settingStore,
        designStore,
        togNavTheme,
        togNavMode,
        togTheme,
        darkTheme: theme.darkAlgorithm,
        openDrawer,
        closeDrawer,
        animateOptions,
        directionsOptions,
      };
    },
  });
</script>

<style lang="less" scoped>
  .drawer {
    .ant-divider:not(.ant-divider-vertical) {
      margin: 10px 0;
    }

    &-setting-item {
      display: flex;
      align-items: center;
      padding: 12px 0;
      flex-wrap: wrap;

      &-style {
        display: inline-block;
        position: relative;
        margin-right: 16px;
        cursor: pointer;
        text-align: center;
      }

      &-title {
        flex: 1 1;
        font-size: 14px;
      }

      &-action {
        flex: 0 0 auto;
      }

      &-select {
        flex: 1;
      }

      .theme-item {
        width: 20px;
        min-width: 20px;
        height: 20px;
        cursor: pointer;
        border: 1px solid #eee;
        border-radius: 2px;
        margin: 0 5px 5px 0;
        text-align: center;
        line-height: 14px;

        .theme-check-icon {
          color: #fff;
          font-size: 12px;
        }
      }
    }

    .align-items-top {
      align-items: flex-start;
      padding: 2px 0;
    }

    .justify-center {
      justify-content: center;
    }

    .dark-switch :deep(.ant-switch) {
      background-color: #000e1c;
    }
  }
</style>
