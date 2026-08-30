/**
 * 渲染服务 Vue Composable
 *
 * 提供在 Vue 组件中使用渲染服务的便捷方式
 */

import { ref, onMounted, onUnmounted, computed, type ComputedRef } from 'vue';
import {
  RenderService,
  type RenderResult,
  type TemplateFile,
  type EngineConfig,
  type EngineManagerEvent,
} from '@/services';

export interface UseRenderServiceOptions {
  /** 引擎配置 */
  config?: Partial<EngineConfig>;
  /** 是否自动初始化 */
  autoInit?: boolean;
}

export interface UseRenderServiceReturn {
  /** 是否已初始化 */
  isReady: ReturnType<typeof ref<boolean>>;
  /** 当前使用的引擎名称 */
  currentEngine: ReturnType<typeof ref<string>>;
  /** 当前引擎版本 */
  engineVersion: ReturnType<typeof ref<string>>;
  /** 是否正在加载 */
  isLoading: ReturnType<typeof ref<boolean>>;
  /** 错误信息 */
  error: ReturnType<typeof ref<string | null>>;
  /** 是否使用 WASM 引擎 */
  isUsingWasm: ComputedRef<boolean>;
  /** 渲染模板 */
  render: (template: string, variables: Record<string, unknown>) => Promise<RenderResult>;
  /** 渲染指定模板文件（通过 templateId + filePath） */
  renderFile: (
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ) => Promise<RenderResult>;
  /** 渲染文件树 */
  renderTree: (
    templateId: number,
    files: TemplateFile[],
    variables: Record<string, unknown>
  ) => Promise<RenderResult[]>;
  /** 切换引擎 */
  switchEngine: (engineType: 'wasm' | 'backend') => Promise<boolean>;
  /** 清除 WASM 缓存 */
  clearWasmCache: () => Promise<void>;
  /** 初始化服务 */
  initialize: () => Promise<void>;
  /** 获取服务状态 */
  getStatus: () => { currentEngine: string; wasmReady: boolean; backendReady: boolean };
}

/**
 * 渲染服务 Composable
 *
 * @param options - 配置选项
 * @returns 渲染服务接口
 *
 * @example
 * ```vue
 * <script setup>
 * import { useRenderService } from '@/composables/useRenderService';
 *
 * const { isReady, currentEngine, render, isUsingWasm } = useRenderService();
 *
 * async function handleRender() {
 *   const result = await render('Hello {{ name }}!', { name: 'World' });
 *   console.log(result.content);
 * }
 * </script>
 *
 * <template>
 *   <div>
 *     <span>Engine: {{ currentEngine }}</span>
 *     <span v-if="isUsingWasm"> (Offline Ready)</span>
 *   </div>
 * </template>
 * ```
 */
export function useRenderService(options: UseRenderServiceOptions = {}): UseRenderServiceReturn {
  const { config, autoInit = true } = options;

  // 响应式状态
  const isReady = ref(false);
  const currentEngine = ref('');
  const engineVersion = ref('');
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // 获取渲染服务实例
  const service = RenderService.getInstance(config);

  // 计算属性
  const isUsingWasm = computed(() => currentEngine.value === 'WASM');

  // 事件监听器清理函数
  let cleanupListener: (() => void) | null = null;

  /**
   * 初始化服务
   */
  async function initialize(): Promise<void> {
    if (isReady.value) return;

    isLoading.value = true;
    error.value = null;

    try {
      await service.initialize();
      isReady.value = true;
      currentEngine.value = service.getCurrentEngine();
      engineVersion.value = service.getCurrentEngineVersion();

      // 监听引擎切换事件
      cleanupListener = service.addEventListener((event: EngineManagerEvent) => {
        switch (event.type) {
          case 'engine:switched':
            currentEngine.value = event.to;
            break;
          case 'engine:error':
            error.value = event.error;
            break;
        }
      });
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to initialize';
      console.error('[useRenderService] Initialization failed:', e);
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 渲染模板字符串
   */
  async function render(
    template: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult> {
    if (!isReady.value) {
      await initialize();
    }

    return service.render(template, variables);
  }

  /**
   * 渲染指定模板文件
   */
  async function renderFile(
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult> {
    if (!isReady.value) {
      await initialize();
    }

    return service.renderFile(templateId, filePath, variables);
  }

  /**
   * 批量渲染文件树
   */
  async function renderTree(
    templateId: number,
    files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]> {
    if (!isReady.value) {
      await initialize();
    }

    return service.renderTree(templateId, files, variables);
  }

  /**
   * 切换引擎
   */
  async function switchEngine(engineType: 'wasm' | 'backend'): Promise<boolean> {
    const success = await service.switchEngine(engineType);
    if (success) {
      currentEngine.value = service.getCurrentEngine();
    }
    return success;
  }

  /**
   * 清除 WASM 缓存
   */
  async function clearWasmCache(): Promise<void> {
    await service.clearWasmCache();
    currentEngine.value = service.getCurrentEngine();
  }

  /**
   * 获取服务状态
   */
  function getStatus(): { currentEngine: string; wasmReady: boolean; backendReady: boolean } {
    return service.getStatus();
  }

  // 生命周期
  onMounted(async () => {
    if (autoInit) {
      await initialize();
    }
  });

  onUnmounted(() => {
    if (cleanupListener) {
      cleanupListener();
      cleanupListener = null;
    }
  });

  return {
    isReady,
    currentEngine,
    engineVersion,
    isLoading,
    error,
    isUsingWasm,
    render,
    renderFile,
    renderTree,
    switchEngine,
    clearWasmCache,
    initialize,
    getStatus,
  };
}

export default useRenderService;
