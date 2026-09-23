import './assets/styles/variables.css'
import './assets/styles/themes.css'
import 'ant-design-vue/dist/reset.css'

import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import pinia from './stores'
import Antd from 'ant-design-vue'
import { applyDisplay } from './composables/useDisplayPrefs'

const app = createApp(App)

app.use(pinia)
app.use(router)
app.use(Antd)

// 启动即应用显示偏好(主题色/字号/紧凑/动画),不等设置页打开
applyDisplay()

app.mount('#app')
