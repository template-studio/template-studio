/**
 * 引擎管理器（桌面版）
 *
 * 负责管理和切换本地引擎（Tauri 原生，与 web 版的 WASM 同槽位）与后端引擎。
 *
 * 兼容说明：为使 web 编辑器视图零改动复用，公共 API 沿用 web 版的
 * wasm 命名（preferredEngine 'wasm'、isUsingWasm、clearWasmCache）——
 * 在桌面端这些均指本地 Tauri 引擎。唯一例外是 AdvancedDrawer 中
 * `currentEngine === 'WASM'` 的字面量判断，移植时已随视图调整。
 */

import type {
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
import { TauriEngine } from './TauriEngine';
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
  private localEngine: TauriEngine;
  private backendEngine: BackendEngine;
  private currentEngine: TauriEngine | BackendEngine;
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

    this.localEngine = new TauriEngine();
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
          this.currentEngine = this.localEngine;
        } else {
          console.warn('[EngineManager] 本地引擎不可用，回退后端引擎');
          this.currentEngine = this.backendEngine;
        }
        break;

      case 'backend':
        this.currentEngine = this.backendEngine;
        break;

      case 'auto':
      default:
        // 优先使用本地引擎（离线可用）
        if (states.wasm.available) {
          this.currentEngine = this.localEngine;
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
      wasm: { engine: this.localEngine, available: false },
      backend: { engine: this.backendEngine, available: true },
    };

    // 检查本地（Tauri）引擎
    try {
      await this.localEngine.initialize();
      states.wasm.available = this.localEngine.isReady;

      if (states.wasm.available) {
        this.emit({
          type: 'wasm:loaded',
          source: 'cache', // 桌面端本地引擎无缓存概念，仅为兼容事件类型
        });
      }
    } catch (error) {
      states.wasm.available = false;
      states.wasm.error = error instanceof Error ? error.message : 'Unknown error';
      console.warn('[EngineManager] 本地引擎不可用:', states.wasm.error);

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
  getCurrentEngine() {
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
   * 检查是否使用本地（Tauri）引擎
   */
  isUsingWasm(): boolean {
    return this.currentEngine === this.localEngine;
  }

  /**
   * 检查本地引擎是否可用
   */
  async isWasmAvailable(): Promise<boolean> {
    try {
      if (!this.localEngine.isReady) {
        await this.localEngine.initialize();
      }
      return this.localEngine.isReady;
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
      if (!this.localEngine.isReady) {
        try {
          await this.localEngine.initialize();
        } catch (error) {
          console.error('[EngineManager] 切换本地引擎失败:', error);
          this.emit({
            type: 'engine:error',
            engine: 'wasm',
            error: error instanceof Error ? error.message : 'Unknown error',
          });
          return false;
        }
      }

      if (!this.localEngine.isReady) {
        return false;
      }

      this.currentEngine = this.localEngine;
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

    // 如果当前引擎是本地引擎且未就绪，回退到后端
    if (this.currentEngine === this.localEngine && !this.localEngine.isReady) {
      console.warn('[EngineManager] 本地引擎未就绪，回退后端引擎');
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

    // 如果当前引擎是本地引擎且未就绪，回退到后端
    if (this.currentEngine === this.localEngine && !this.localEngine.isReady) {
      console.warn('[EngineManager] 本地引擎未就绪，回退后端引擎');
      return this.backendEngine.renderTree(templateId, files, variables);
    }

    return this.currentEngine.renderTree(templateId, files, variables);
  }

  /**
   * 渲染指定模板文件
   *
   * 通过 templateId + filePath 从后端读取文件并渲染，支持文件依赖。
   * 本地引擎不支持此模式，自动走后端引擎。
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
    return this.backendEngine.renderFile(templateId, filePath, variables);
  }

  /**
   * 清除本地引擎缓存（桌面端为空操作，保留 API 兼容）
   */
  async clearWasmCache(): Promise<void> {
    // TauriEngine 无前端缓存；Rust 侧模板集缓存按内容哈希自动失效
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
      wasmReady: this.localEngine.isReady,
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
