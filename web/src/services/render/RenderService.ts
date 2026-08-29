/**
 * 渲染服务
 *
 * 提供统一的模板渲染接口，是应用层使用的主要入口
 */

import type {
  RenderResult,
  TemplateFile,
  EngineConfig,
  EngineInfo,
  RenderStats,
  EngineManagerEventListener,
} from '../types';
import { EngineManager } from './EngineManager';

/**
 * 渲染服务类
 *
 * 单例模式，提供全局的渲染功能
 */
export class RenderService {
  private static instance: RenderService | null = null;
  private manager: EngineManager;

  private constructor(config?: Partial<EngineConfig>) {
    this.manager = new EngineManager(config);
  }

  /**
   * 获取单例实例
   */
  static getInstance(config?: Partial<EngineConfig>): RenderService {
    if (!RenderService.instance) {
      RenderService.instance = new RenderService(config);
    }
    return RenderService.instance;
  }

  /**
   * 重置单例（用于测试或重新配置）
   */
  static resetInstance(): void {
    RenderService.instance = null;
  }

  /**
   * 初始化服务
   */
  async initialize(): Promise<void> {
    await this.manager.initialize();
  }

  /**
   * 渲染模板字符串
   */
  async render(template: string, variables: Record<string, unknown>): Promise<RenderResult> {
    return this.manager.render(template, variables);
  }

  /**
   * 渲染指定模板文件
   *
   * 通过 templateId + filePath 从后端读取文件并渲染，支持文件依赖。
   */
  async renderFile(
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult> {
    return this.manager.renderFile(templateId, filePath, variables);
  }

  /**
   * 批量渲染文件树
   */
  async renderTree(
    templateId: number,
    files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]> {
    return this.manager.renderTree(templateId, files, variables);
  }

  /**
   * 获取当前使用的引擎名称
   */
  getCurrentEngine(): string {
    return this.manager.getCurrentEngineName();
  }

  /**
   * 获取当前引擎版本
   */
  getCurrentEngineVersion(): string {
    return this.manager.getCurrentEngineVersion();
  }

  /**
   * 检查是否使用 WASM 引擎
   */
  isUsingWasm(): boolean {
    return this.manager.isUsingWasm();
  }

  /**
   * 切换引擎
   */
  async switchEngine(engineType: 'wasm' | 'backend'): Promise<boolean> {
    return this.manager.switchEngine(engineType);
  }

  /**
   * 清除 WASM 缓存
   */
  async clearWasmCache(): Promise<void> {
    await this.manager.clearWasmCache();
  }

  /**
   * 添加事件监听器
   */
  addEventListener(listener: EngineManagerEventListener): () => void {
    return this.manager.addEventListener(listener);
  }

  /**
   * 获取渲染统计信息
   */
  getStats(): RenderStats {
    return {
      engine: this.manager.getCurrentEngineName(),
      duration: 0, // 实际耗时由每次渲染返回
    };
  }

  /**
   * 获取引擎状态
   */
  getStatus(): {
    currentEngine: string;
    wasmReady: boolean;
    backendReady: boolean;
  } {
    return this.manager.getStatus();
  }

  /**
   * 获取当前引擎的详细信息
   */
  async getCurrentEngineInfo(): Promise<EngineInfo | null> {
    return this.manager.getCurrentEngineInfo();
  }
}

// 便捷导出
export default RenderService;
