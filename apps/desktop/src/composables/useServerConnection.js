import { ref } from 'vue'
import { useConfigStore } from '@/stores/config'

/**
 * 服务端连接状态感知(#716)
 *
 * 轻量探活:周期 GET 引擎信息公开端点(免鉴权、开销小),连续失败阈值判定离线。
 * App.vue 顶部消费 offline 状态渲染横幅;恢复在线时自动消隐。
 */

const OFFLINE_THRESHOLD = 2 // 连续失败次数判定离线(容忍单次抖动)
const INTERVAL_MS = 30000   // 探活周期

const offline = ref(false)
let started = false
let fails = 0
let timer = null

async function probe() {
  const configStore = useConfigStore()
  const base = (configStore.configLoaded ? configStore.baseURL : 'http://127.0.0.1:8080').replace(/\/+$/, '')
  try {
    // 任意 HTTP 响应(含 4xx/5xx)都说明网络与服务在,只判 fetch 抛错
    await fetch(`${base}/api/v1/engine/info`, { signal: AbortSignal.timeout(5000) })
    if (offline.value) console.info('[conn] 服务端已恢复在线')
    fails = 0
    offline.value = false
  } catch {
    fails++
    if (fails >= OFFLINE_THRESHOLD && !offline.value) {
      offline.value = true
      console.warn('[conn] 服务端不可达,已判定离线')
    }
  }
}

export function useServerConnection() {
  const start = () => {
    if (started) return
    started = true
    probe()
    timer = setInterval(probe, INTERVAL_MS)
  }
  const stop = () => {
    if (timer) clearInterval(timer)
    timer = null
    started = false
  }
  return { offline, start, stop, probeNow: probe }
}
