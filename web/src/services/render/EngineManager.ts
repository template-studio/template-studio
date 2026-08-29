/**
 * 引擎管理器
 *
 * 负责管理和切换 WASM 引擎与后端引擎
 */

import type {
  RenderEngine,
  RenderResult,
  TemplateFile,
  EngineConfig,
  EngineState,
  EngineInfo,
  EngineManagerEvent,
  EngineManagerEventListener,
  EnginePreference,
} from '../types';
import { DEFAULT_ENGINE_CONFIG } from '../types';
import { WasmEngine } from './WasmEngine';
import { BackendEngine } from './BackendEngine';

type EngineType = 'wasm' | 'backend';

// localStorage key for engine preference
const ENGINE_PREFERENCE_KEY = 'template-studio-engine-preference';

/**
 * 引擎管理器类
 *
 * 提供统一的渲染接口，自动选择最优引擎
 */
export class EngineManager {
  private wasmEngine: WasmEngine;
  private backendEngine: BackendEngine;
  private currentEngine: RenderEngine;
  private config: EngineConfig;
  private listeners: Set<EngineManagerEventListener> = new Set();
  private initialized = false;

  constructor(config: Partial<EngineConfig> = {}) {
    // 从 localStorage 读取保存的偏好设置
    const savedPreference = this.getSavedPreference();

    this.config = {
      ...DEFAULT_ENGINE_CONFIG,
      ...config,
      // 优先使用保存的偏好设置
      preferredEngine:
        savedPreference || config.preferredEngine || DEFAULT_ENGINE_CONFIG.preferredEngine,
    };

    this.wasmEngine = new WasmEngine();
    this.backendEngine = new BackendEngine();
    this.currentEngine = this.backendEngine; // 默认使用后端引擎
  }

  /**
   * 从 localStorage 获取保存的引擎偏好
   */
  private getSavedPreference(): EnginePreference | null {
    try {
      const saved = localStorage.getItem(ENGINE_PREFERENCE_KEY);
      if (saved === 'wasm' || saved === 'backend' || saved === 'auto') {
        return saved;
      }
    } catch {
      // ignore
    }
    return null;
  }

  /**
   * 保存引擎偏好到 localStorage
   */
  private savePreference(preference: EnginePreference): void {
    try {
      localStorage.setItem(ENGINE_PREFERENCE_KEY, preference);
    } catch {
      // ignore
    }
  }

  /**
   * 初始化引擎管理器
   *
   * 根据配置选择并初始化首选引擎
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    console.log('[EngineManager] Initializing...');

    const states = await this.checkEngines();

    // 根据配置选择引擎
    switch (this.config.preferredEngine) {
      case 'wasm':
        if (states.wasm.available) {
          this.currentEngine = this.wasmEngine;
        } else {
          console.warn('[EngineManager] WASM not available, falling back to backend');
          this.currentEngine = this.backendEngine;
        }
        break;

      case 'backend':
        this.currentEngine = this.backendEngine;
        break;

      case 'auto':
      default:
        // 优先使用 WASM（离线可用）
        if (states.wasm.available) {
          this.currentEngine = this.wasmEngine;
        } else {
          this.currentEngine = this.backendEngine;
        }
        break;
    }

    this.initialized = true;
    this.emit({ type: 'engine:initialized', engine: this.currentEngine.name });

    console.log(
      `[EngineManager] Using engine: ${this.currentEngine.name} (v${this.currentEngine.version})`
    );
  }

  /**
   * 检查所有引擎的可用状态
   */
  private async checkEngines(): Promise<Record<EngineType, EngineState>> {
    const states: Record<EngineType, EngineState> = {
      wasm: { engine: this.wasmEngine, available: false },
      backend: { engine: this.backendEngine, available: true },
    };

    // 检查 WASM 引擎
    try {
      await this.wasmEngine.initialize();
      states.wasm.available = this.wasmEngine.isReady;

      if (states.wasm.available) {
        this.emit({
          type: 'wasm:loaded',
          source: 'cache', // 实际来源由 WasmEngine 内部决定
        });
      }
    } catch (error) {
      states.wasm.available = false;
      states.wasm.error = error instanceof Error ? error.message : 'Unknown error';
      console.warn('[EngineManager] WASM engine unavailable:', states.wasm.error);

      this.emit({
        type: 'engine:error',
        engine: 'wasm',
        error: states.wasm.error,
      });
    }

    // 检查后端引擎
    try {
      await this.backendEngine.initialize();
      states.backend.available = this.backendEngine.isReady;
    } catch (error) {
      states.backend.available = false;
      states.backend.error = error instanceof Error ? error.message : 'Unknown error';
    }

    return states;
  }

  /**
   * 获取当前使用的引擎
   */
  getCurrentEngine(): RenderEngine {
    return this.currentEngine;
  }

  /**
   * 获取当前引擎名称
   */
  getCurrentEngineName(): string {
    return this.currentEngine.name;
  }

  /**
   * 获取当前引擎版本
   */
  getCurrentEngineVersion(): string {
    return this.currentEngine.version;
  }

  /**
   * 检查是否使用 WASM 引擎
   */
  isUsingWasm(): boolean {
    return this.currentEngine === this.wasmEngine;
  }

  /**
   * 检查 WASM 引擎是否可用
   */
  async isWasmAvailable(): Promise<boolean> {
    try {
      if (!this.wasmEngine.isReady) {
        await this.wasmEngine.initialize();
      }
      return this.wasmEngine.isReady;
    } catch {
      return false;
    }
  }

  /**
   * 手动切换引擎
   */
  async switchEngine(engineType: EngineType): Promise<boolean> {
    const previousEngine = this.currentEngine.name;

    if (engineType === 'wasm') {
      if (!this.wasmEngine.isReady) {
        try {
          await this.wasmEngine.initialize();
        } catch (error) {
          console.error('[EngineManager] Failed to switch to WASM:', error);
          this.emit({
            type: 'engine:error',
            engine: 'wasm',
            error: error instanceof Error ? error.message : 'Unknown error',
          });
          return false;
        }
      }

      if (!this.wasmEngine.isReady) {
        return false;
      }

      this.currentEngine = this.wasmEngine;
    } else {
      this.currentEngine = this.backendEngine;
    }

    // 保存偏好设置到 localStorage
    this.savePreference(engineType);
    this.config.preferredEngine = engineType;

    this.emit({
      type: 'engine:switched',
      from: previousEngine,
      to: this.currentEngine.name,
    });

    console.log(`[EngineManager] Switched to ${this.currentEngine.name}`);
    return true;
  }

  /**
   * 渲染模板字符串
   *
   * 如果当前引擎不可用，自动回退到备用引擎
   */
  async render(template: string, variables: Record<string, unknown>): Promise<RenderResult> {
    // 确保已初始化
    if (!this.initialized) {
      await this.initialize();
    }

    // 如果当前引擎是 WASM 且未就绪，回退到后端
    if (this.currentEngine === this.wasmEngine && !this.wasmEngine.isReady) {
      console.warn('[EngineManager] WASM not ready, falling back to backend');
      return this.backendEngine.render(template, variables);
    }

    return this.currentEngine.render(template, variables);
  }

  /**
   * 批量渲染文件树
   *
   * 如果当前引擎不可用，自动回退到备用引擎
   */
  async renderTree(
    templateId: number,
    files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]> {
    // 确保已初始化
    if (!this.initialized) {
      await this.initialize();
    }

    // 如果当前引擎是 WASM 且未就绪，回退到后端
    if (this.currentEngine === this.wasmEngine && !this.wasmEngine.isReady) {
      console.warn('[EngineManager] WASM not ready, falling back to backend');
      return this.backendEngine.renderTree(templateId, files, variables);
    }

    return this.currentEngine.renderTree(templateId, files, variables);
  }

  /**
   * 渲染指定模板文件
   *
   * 通过 templateId + filePath 从后端读取文件并渲染，支持文件依赖。
   * WASM 引擎不支持此模式，会自动回退到后端引擎。
   */
  async renderFile(
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult> {
    // 确保已初始化
    if (!this.initialized) {
      await this.initialize();
    }

    // 文件模式需要使用后端引擎（支持文件依赖）
    // WASM 引擎不支持文件模式，直接使用后端
    return this.backendEngine.renderFile(templateId, filePath, variables);
  }

  /**
   * 清除 WASM 缓存
   */
  async clearWasmCache(): Promise<void> {
    await this.wasmEngine.clearCache();

    // 如果当前使用的是 WASM，需要重新初始化
    if (this.currentEngine === this.wasmEngine) {
      this.initialized = false;
      await this.initialize();
    }
  }

  /**
   * 添加事件监听器
   */
  addEventListener(listener: EngineManagerEventListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * 发送事件
   */
  private emit(event: EngineManagerEvent): void {
    this.listeners.forEach((listener) => {
      try {
        listener(event);
      } catch (error) {
        console.error('[EngineManager] Event listener error:', error);
      }
    });
  }

  /**
   * 获取引擎状态信息
   */
  getStatus(): {
    currentEngine: string;
    wasmReady: boolean;
    backendReady: boolean;
    config: EngineConfig;
  } {
    return {
      currentEngine: this.currentEngine.name,
      wasmReady: this.wasmEngine.isReady,
      backendReady: this.backendEngine.isReady,
      config: this.config,
    };
  }

  /**
   * 获取当前引擎的详细信息
   */
  async getCurrentEngineInfo(): Promise<EngineInfo | null> {
    return this.currentEngine.getInfo();
  }
}
