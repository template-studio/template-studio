/**
 * WASM 引擎实现
 *
 * 提供浏览器端的模板渲染功能，支持离线使用
 */

import type { RenderEngine, RenderResult, TemplateFile, EngineInfo } from '../types';

// 从 src/wasm 导入 WASM 模块
// @ts-ignore - Vite 会处理 WASM 导入
import init, {
  render_string as wasm_render_string,
  render_tree as wasm_render_tree,
  get_engine_info as wasm_get_engine_info,
  clear_cache as wasm_clear_cache,
  get_cache_size as wasm_get_cache_size,
} from '@/wasm/template_studio_template_core_wasm.js';

// WASM 模块类型定义
interface WasmRenderResult {
  content: string;
  success: boolean;
  error?: {
    error_type: string;
    message: string;
    line?: number;
    column?: number;
    context?: string;
    suggestion?: string;
  };
}

interface WasmRenderedFile {
  id: number;
  file_path: string;
  file_name: string;
  file_content?: string;
  is_directory: number;
  filesize: number;
  parent_id: number;
  error?: {
    error_type: string;
    message: string;
    line?: number;
    column?: number;
    context?: string;
    suggestion?: string;
  };
}

// 全局 WASM 初始化状态
let wasmInitialized = false;
let wasmInitPromise: Promise<void> | null = null;

/**
 * WASM 引擎类
 *
 * 负责加载、缓存和使用 WASM 模板渲染引擎
 */
export class WasmEngine implements RenderEngine {
  readonly name = 'WASM';

  private _version = 'unknown';
  private info: EngineInfo | null = null;
  private initPromise: Promise<void> | null = null;

  constructor() {
    // WASM 模块从 src/wasm 目录加载，不需要 IndexedDB 缓存
  }

  get version(): string {
    return this._version;
  }

  get isReady(): boolean {
    return wasmInitialized;
  }

  /**
   * 初始化 WASM 引擎
   */
  async initialize(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }

    this.initPromise = this.doInitialize();
    return this.initPromise;
  }

  private async doInitialize(): Promise<void> {
    console.log('[WasmEngine] Initializing...');

    // 如果已经初始化，直接返回
    if (wasmInitialized) {
      this._version = await this.getVersion();
      console.log(`[WasmEngine] Already initialized (version: ${this._version})`);
      return;
    }

    // 如果正在初始化，等待完成
    if (wasmInitPromise) {
      await wasmInitPromise;
      this._version = await this.getVersion();
      return;
    }

    wasmInitPromise = (async () => {
      try {
        // 初始化 WASM 模块
        await init();
        wasmInitialized = true;
        console.log('[WasmEngine] WASM module initialized');
      } catch (error) {
        wasmInitPromise = null;
        throw error;
      }
    })();

    await wasmInitPromise;
    this._version = await this.getVersion();
    console.log(`[WasmEngine] Loaded successfully (version: ${this._version})`);
  }

  /**
   * 获取版本信息
   */
  private async getVersion(): Promise<string> {
    try {
      const info = await this.getInfo();
      return info?.version || 'unknown';
    } catch {
      return 'unknown';
    }
  }

  /**
   * 渲染模板字符串
   */
  async render(template: string, variables: Record<string, unknown>): Promise<RenderResult> {
    if (!wasmInitialized) {
      await this.initialize();
    }

    const startTime = performance.now();

    try {
      const result = wasm_render_string(template, variables) as WasmRenderResult;

      return {
        content: result.content,
        success: result.success,
        error: result.error
          ? {
              type: this.mapErrorType(result.error.error_type),
              message: result.error.message,
              line: result.error.line,
              column: result.error.column,
              context: result.error.context,
              suggestion: result.error.suggestion,
            }
          : undefined,
        duration: Math.round(performance.now() - startTime),
      };
    } catch (error) {
      return {
        content: '',
        success: false,
        error: {
          type: 'runtime',
          message: error instanceof Error ? error.message : 'Unknown error',
        },
        duration: Math.round(performance.now() - startTime),
      };
    }
  }

  /**
   * 渲染指定模板文件
   *
   * 注意：WASM 引擎不支持通过 templateId + filePath 读取文件，
   * 需要先获取文件内容，然后使用 render 方法。
   * 此方法返回错误，建议调用者回退到后端引擎或先获取文件内容。
   */
  async renderFile(
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult> {
    // WASM 引擎不支持文件模式，返回错误让调用者回退到后端
    return {
      content: '',
      success: false,
      error: {
        type: 'not_found',
        message: 'WASM 引擎不支持文件模式，请使用后端引擎或先获取文件内容',
      },
      duration: 0,
    };
  }

  /**
   * 批量渲染文件树
   */
  async renderTree(
    files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]> {
    if (!wasmInitialized) {
      await this.initialize();
    }

    const startTime = performance.now();

    try {
      const wasmFiles = files.map((f) => ({
        id: 0,
        file_path: f.path,
        file_name: f.path.split('/').pop() || f.path,
        file_content: f.content,
        is_directory: 0,
        parent_id: 0,
        filesize: f.content.length,
      }));

      const results = wasm_render_tree(wasmFiles, variables) as WasmRenderedFile[];

      return results.map((r) => ({
        path: r.file_path,
        content: r.file_content || '',
        success: !r.error,
        error: r.error
          ? {
              type: this.mapErrorType(r.error.error_type),
              message: r.error.message,
              line: r.error.line,
              column: r.error.column,
              context: r.error.context,
              suggestion: r.error.suggestion,
            }
          : undefined,
        duration: Math.round(performance.now() - startTime),
      }));
    } catch (error) {
      const duration = Math.round(performance.now() - startTime);
      return files.map(() => ({
        content: '',
        success: false,
        error: {
          type: 'runtime',
          message: error instanceof Error ? error.message : 'Unknown error',
        },
        duration,
      }));
    }
  }

  /**
   * 映射错误类型
   */
  private mapErrorType(type: string): 'syntax' | 'runtime' | 'network' | 'not_found' | 'unknown' {
    switch (type) {
      case 'parse_error':
      case 'syntax_error':
        return 'syntax';
      case 'variable_error':
      case 'filter_error':
      case 'render_error':
        return 'runtime';
      case 'not_found':
        return 'not_found';
      default:
        return 'unknown';
    }
  }

  /**
   * 获取引擎信息
   */
  async getInfo(): Promise<EngineInfo | null> {
    if (!wasmInitialized) {
      return null;
    }

    try {
      const info = wasm_get_engine_info();
      return {
        version: info.version,
        buildTime: info.build_time,
        size: 0,
        filters: info.filters.map((f: { name: string }) => f.name),
        functions: info.functions,
      };
    } catch {
      return this.info;
    }
  }

  /**
   * 清除缓存
   */
  async clearCache(): Promise<void> {
    if (wasmInitialized) {
      wasm_clear_cache();
    }
    wasmInitialized = false;
    wasmInitPromise = null;
    this.initPromise = null;
    console.log('[WasmEngine] Cache cleared');
  }

  /**
   * 获取缓存大小
   */
  getCacheSize(): number {
    return wasmInitialized ? wasm_get_cache_size() : 0;
  }
}
