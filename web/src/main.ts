import './styles/tailwind.css';
import './styles/index.less';
import { createApp } from 'vue';
import { setupAntdDiscreteApi, setupAntd, setupDirectives } from '@/plugins';
import App from './App.vue';
import router, { setupRouter } from './router';
import { setupStore } from '@/store';

async function bootstrap() {
  const app = createApp(App);

  // 挂载状态管理
  setupStore(app);

  // 注册全局 Ant Design Vue 组件
  setupAntd(app);

  // 挂载 Ant Design Vue 脱离上下文的 Api
  setupAntdDiscreteApi();

  // 注册全局自定义组件
  //setupCustomComponents();

  // 注册全局自定义指令，如：v-permission权限指令
  setupDirectives(app);

  // 注册全局方法，如：app.config.globalProperties.$message = message
  //setupGlobalMethods(app);

  // 挂载路由
  setupRouter(app);

  // 路由准备就绪后挂载 APP 实例
  // https://router.vuejs.org/api/interfaces/router.html#isready
  await router.isReady();

  app.mount('#app', true);
}

void bootstrap();
