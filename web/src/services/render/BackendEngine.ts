/**
 * 后端引擎实现
 *
 * 通过 API 调用后端服务进行模板渲染
 */

import type { RenderEngine, RenderResult, TemplateFile, EngineInfo } from '../types';

/**
 * 获取后端 API 基础 URL
 */
function getBackendBaseUrl(): string {
  // 1. 优先使用环境变量配置
  const apiUrl = import.meta.env.VITE_API_URL;
  if (apiUrl) {
    // 移除末尾斜杠
    return apiUrl.replace(/\/$/, '');
  }

  // 2. 开发环境默认使用后端端口
  if (import.meta.env.DEV) {
    return 'http://localhost:8080';
  }

  // 3. 生产环境使用当前域名
  const { protocol, hostname, port } = window.location;
  return `${protocol}//${hostname}${port ? `:${port}` : ''}`;
}

/**
 * 后端引擎类
 *
 * 当 WASM 引擎不可用时的备选方案
 */
export class BackendEngine implements RenderEngine {
  readonly name = 'Backend';
  readonly version: string;

  private ready = true;
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.version = 'server';
    // 使用传入的 baseUrl 或自动获取完整后端 URL
    this.baseUrl = baseUrl || `${getBackendBaseUrl()}/api/v1`;
  }

  get isReady(): boolean {
    return this.ready;
  }

  /**
   * 初始化引擎
   *
   * 后端引擎不需要特殊初始化，只需检查网络连接
   */
  async initialize(): Promise<void> {
    try {
      // 检查后端服务是否可用
      const response = await fetch(`${this.baseUrl}/engine/info`, {
        method: 'HEAD',
      });

      this.ready = response.ok;

      if (!this.ready) {
        console.warn('[BackendEngine] Backend service not available');
      }
    } catch (error) {
      console.warn('[BackendEngine] Failed to connect to backend:', error);
      this.ready = true; // 仍然标记为就绪，让实际请求时处理错误
    }
  }

  /**
   * 渲染模板字符串（直接传入内容）
   * 注意：此方法不支持文件依赖，建议使用 renderFile 方法
   */
  async render(template: string, variables: Record<string, unknown>): Promise<RenderResult> {
    const startTime = performance.now();

    try {
      const response = await fetch(`${this.baseUrl}/template-files/preview`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          content: template,
          variables,
        }),
      });

      const duration = Math.round(performance.now() - startTime);

      if (!response.ok) {
        return {
          content: '',
          success: false,
          error: {
            type: 'network',
            message: `HTTP error: ${response.status} ${response.statusText}`,
          },
          duration,
        };
      }

      const result = await response.json();

      if (result.code !== 0) {
        return {
          content: '',
          success: false,
          error: {
            type: 'runtime',
            message: result.message || 'Unknown error',
          },
          duration,
        };
      }

      return {
        content: result.data?.fileContent || result.data?.content || '',
        success: true,
        duration,
      };
    } catch (error) {
      return {
        content: '',
        success: false,
        error: {
          type: 'network',
          message: error instanceof Error ? error.message : 'Network error',
        },
        duration: Math.round(performance.now() - startTime),
      };
    }
  }

  /**
   * 渲染指定模板文件
   *
   * 通过 templateId 和 filePath 从后端读取文件并渲染，支持文件依赖
   */
  async renderFile(
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult> {
    const startTime = performance.now();

    try {
      const response = await fetch(`${this.baseUrl}/template-files/preview`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          templateId,
          filePath,
          variables,
        }),
      });

      const duration = Math.round(performance.now() - startTime);

      if (!response.ok) {
        return {
          content: '',
          success: false,
          error: {
            type: 'network',
            message: `HTTP error: ${response.status} ${response.statusText}`,
          },
          duration,
        };
      }

      const result = await response.json();

      if (result.code !== 0) {
        return {
          content: '',
          success: false,
          error: {
            type: 'runtime',
            message: result.message || 'Unknown error',
          },
          duration,
        };
      }

      // 检查是否有错误信息
      if (result.data?.error) {
        return {
          content: '',
          success: false,
          error: {
            type: 'runtime',
            message: result.data.error.message || 'Render error',
            line: result.data.error.line,
            column: result.data.error.column,
            context: result.data.error.context,
          },
          duration,
        };
      }

      return {
        content: result.data?.fileContent || '',
        success: true,
        duration,
      };
    } catch (error) {
      return {
        content: '',
        success: false,
        error: {
          type: 'network',
          message: error instanceof Error ? error.message : 'Network error',
        },
        duration: Math.round(performance.now() - startTime),
      };
    }
  }

  /**
   * 批量渲染文件树
   *
   * 后端 preview-tree 接口按 templateId 从存储读取模板文件与 conditions.yml
   * 并在服务端完成条件过滤，因此这里只传 templateId 与变量；
   * 返回结果与服务端渲染出的文件树对应（与入参 files 无对应关系）
   */
  async renderTree(
    templateId: number,
    _files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]> {
    const startTime = performance.now();

    const failAll = (type: 'network' | 'runtime', message: string): RenderResult[] => [
      { path: '', content: '', success: false, error: { type, message }, duration: Math.round(performance.now() - startTime) },
    ];

    try {
      const response = await fetch(`${this.baseUrl}/template-files/preview-tree`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          templateId,
          variables,
        }),
      });

      const duration = Math.round(performance.now() - startTime);

      if (!response.ok) {
        return failAll('network', `HTTP error: ${response.status} ${response.statusText}`);
      }

      const result = await response.json();

      if (result.code !== 0) {
        return failAll('runtime', result.message || 'Unknown error');
      }

      // 后端返回嵌套树（data.tree，camelCase 字段），递归展平为渲染结果列表
      const tree: any[] = result.data?.tree || [];
      const flattened: any[] = [];
      const walk = (nodes: any[]) => {
        for (const n of nodes) {
          flattened.push(n);
          if (Array.isArray(n.children)) walk(n.children);
        }
      };
      walk(tree);

      return flattened.map((node) => ({
        path: node.filePath || '',
        content: node.fileContent || '',
        success: !node.renderError,
        error: node.renderError
          ? {
              type: 'runtime' as const,
              message: node.renderError.message || 'Render error',
              line: node.renderError.line,
              column: node.renderError.column,
              context: node.renderError.context,
            }
          : undefined,
        duration,
      }));
    } catch (error) {
      return failAll(
        'network',
        error instanceof Error ? error.message : 'Network error'
      );
    }
  }

  /**
   * 检查后端是否在线
   */
  async checkOnline(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/engine/info`, {
        method: 'HEAD',
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * 获取引擎信息
   */
  async getInfo(): Promise<EngineInfo | null> {
    try {
      const response = await fetch(`${this.baseUrl}/engine/info`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        return null;
      }

      const result = await response.json();

      if (result.code !== 0 || !result.data) {
        return null;
      }

      const data = result.data;
      return {
        version: data.version || 'unknown',
        buildTime: data.build_time || '',
        size: data.size || 0,
        filters: data.filters?.map((f: { name: string }) => f.name) || [],
        functions: data.functions || [],
        checksum: data.checksum,
      };
    } catch (error) {
      console.warn('[BackendEngine] Failed to get engine info:', error);
      return null;
    }
  }
}
