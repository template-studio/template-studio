/**
 * Tauri 原生渲染引擎实现
 *
 * 通过 Tauri 命令调用 template_core（与服务器同 crate 的原生引擎），
 * 渲染编辑器中未保存的内存文件集。桌面端编辑器的主引擎；
 * BackendEngine（服务端渲染，仅覆盖已保存状态）作为兜底。
 *
 * 与 WasmEngine 的差异：错误对象的类型字段为 `type`（core::RenderError 的
 * serde 原生命名），而非 WASM 端包装层的 `error_type`。
 */

import { invoke } from '@tauri-apps/api/core';
import type { RenderEngine, RenderResult, TemplateFile, EngineInfo } from '../types';

// Rust 命令返回类型（core::RenderedFile / core::RenderResult 的 serde 形状，snake_case）
interface CoreRenderedFile {
  id: number;
  file_path: string;
  file_name: string;
  file_content?: string | null;
  is_directory: number;
  filesize: number;
  parent_id: number;
  error?: {
    type: string;
    message: string;
    line?: number;
    column?: number;
    context?: string;
    suggestion?: string;
  } | null;
}

interface CoreRenderResult {
  content: string;
  success: boolean;
  variables?: unknown;
  error?: {
    type: string;
    message: string;
    line?: number;
    column?: number;
    context?: string;
    suggestion?: string;
  } | null;
}

// Rust 命令入参（EditorFile，snake_case）
interface EditorFilePayload {
  id: number;
  file_path: string;
  file_name: string;
  file_content: string;
  is_directory: number;
  parent_id: number;
  filesize: number;
  condition: unknown;
}

export class TauriEngine implements RenderEngine {
  readonly name = 'Tauri';

  private _version = 'unknown';
  private ready = false;
  private initPromise: Promise<void> | null = null;

  get version(): string {
    return this._version;
  }

  get isReady(): boolean {
    return this.ready;
  }

  async initialize(): Promise<void> {
    if (this.initPromise) {
      return this.initPromise;
    }
    this.initPromise = this.doInitialize();
    return this.initPromise;
  }

  private async doInitialize(): Promise<void> {
    try {
      const info = await this.getInfo();
      this._version = info?.version || 'unknown';
      this.ready = true;
    } catch (error) {
      this.ready = false;
      throw error instanceof Error ? error : new Error(String(error));
    }
  }

  /** 渲染单个模板字符串（无文件依赖场景） */
  async render(template: string, variables: Record<string, unknown>): Promise<RenderResult> {
    await this.ensureReady();
    const startTime = performance.now();

    try {
      const result = (await invoke('render_string_content', {
        template,
        variables,
      })) as CoreRenderResult;

      return {
        content: result.content,
        success: result.success,
        error: result.error
          ? {
              type: this.mapErrorType(result.error.type),
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
          message: error instanceof Error ? error.message : String(error),
        },
        duration: Math.round(performance.now() - startTime),
      };
    }
  }

  /**
   * 渲染指定模板文件
   *
   * 原生引擎不按 templateId 读取文件（与 WASM 引擎一致），
   * 调用方应取到文件内容后走 render，或整树走 renderTree。
   */
  async renderFile(
    _templateId: string | number,
    _filePath: string,
    _variables: Record<string, unknown>
  ): Promise<RenderResult> {
    return {
      content: '',
      success: false,
      error: {
        type: 'not_found',
        message: '原生引擎不支持按 ID 读取文件，请使用 renderTree 或先获取文件内容',
      },
      duration: 0,
    };
  }

  /** 批量渲染内存文件树（编辑器未保存内容的实时预览入口） */
  async renderTree(
    _templateId: number,
    files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]> {
    await this.ensureReady();
    const startTime = performance.now();

    try {
      const payload: EditorFilePayload[] = files.map((f) => ({
        id: 0,
        file_path: f.path,
        file_name: f.path.split('/').pop() || f.path,
        file_content: f.content,
        is_directory: 0,
        parent_id: 0,
        filesize: f.content.length,
        // 文件生成条件：条件不满足的文件由 Rust 侧统一过滤（与服务端渲染语义一致）
        condition: f.generateCondition ?? null,
      }));

      const results = (await invoke('render_files', {
        files: payload,
        variables,
      })) as CoreRenderedFile[];

      return results.map((r) => ({
        path: r.file_path,
        content: r.file_content || '',
        success: !r.error,
        error: r.error
          ? {
              type: this.mapErrorType(r.error.type),
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
          message: error instanceof Error ? error.message : String(error),
        },
        duration,
      }));
    }
  }

  /** 获取引擎信息（版本/过滤器/内置函数，来自 template_core） */
  async getInfo(): Promise<EngineInfo | null> {
    try {
      const info = (await invoke('get_render_engine_info')) as {
        version: string;
        buildTime: string;
        filters: string[];
        functions: string[];
      };
      return {
        version: info.version,
        buildTime: info.buildTime,
        size: 0,
        filters: info.filters,
        functions: info.functions,
      };
    } catch {
      return null;
    }
  }

  private async ensureReady(): Promise<void> {
    if (!this.ready) {
      await this.initialize();
    }
  }

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
}
