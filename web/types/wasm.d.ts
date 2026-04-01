/**
 * WASM 模块类型声明
 */

declare module '@/wasm/template_studio_template_core_wasm.js' {
  export interface WasmRenderResult {
    content: string;
    success: boolean;
    error?: {
      type: string;
      message: string;
      line?: number;
      column?: number;
      context?: string;
      suggestion?: string;
    };
  }

  export interface WasmEngineInfo {
    version: string;
    build_time: string;
    filters: Array<{
      name: string;
      description: string;
      example: string;
    }>;
    functions: string[];
  }

  export interface WasmRenderedFile {
    id: number;
    file_path: string;
    file_name: string;
    file_content?: string;
    is_directory: number;
    filesize: number;
    parent_id: number;
    error?: {
      type: string;
      message: string;
      line?: number;
      column?: number;
      context?: string;
    };
  }

  /**
   * 初始化 WASM 模块
   */
  export default function init(moduleOrPath?: unknown): Promise<void>;

  /**
   * 渲染模板字符串
   */
  export function render_string(template: string, variables: unknown): WasmRenderResult;

  /**
   * 批量渲染文件树
   */
  export function render_tree(files: unknown[], variables: unknown): WasmRenderedFile[];

  /**
   * 获取引擎信息
   */
  export function get_engine_info(): WasmEngineInfo;

  /**
   * 获取过滤器列表
   */
  export function get_filters(): Array<{
    name: string;
    description: string;
    example: string;
  }>;

  /**
   * 验证模板语法
   */
  export function validate_template(template: string): { valid: boolean; error?: unknown };

  /**
   * 清除缓存
   */
  export function clear_cache(): void;

  /**
   * 获取缓存大小
   */
  export function get_cache_size(): number;
}
