/**
 * 渲染引擎服务类型定义
 *
 * 定义 WASM 引擎管理功能所需的所有 TypeScript 类型
 */

/**
 * 渲染引擎接口
 *
 * 所有渲染引擎（WASM、Backend）都必须实现此接口
 */
export interface RenderEngine {
  /** 引擎名称标识 */
  readonly name: string;
  /** 引擎版本号 */
  readonly version: string;
  /** 引擎是否已初始化并就绪 */
  readonly isReady: boolean;

  /**
   * 初始化引擎
   *
   * 对于 WASM 引擎，会加载和编译 WASM 模块
   * 对于后端引擎，会检查网络连接
   */
  initialize(): Promise<void>;

  /**
   * 渲染单个模板字符串
   *
   * @param template - 模板内容（支持 Tera/MiniJinja 语法）
   * @param variables - 模板变量对象
   * @returns 渲染结果
   */
  render(template: string, variables: Record<string, unknown>): Promise<RenderResult>;

  /**
   * 渲染指定模板文件
   *
   * @param templateId - 模板 ID
   * @param filePath - 文件路径
   * @param variables - 模板变量对象
   * @returns 渲染结果
   */
  renderFile(
    templateId: string | number,
    filePath: string,
    variables: Record<string, unknown>
  ): Promise<RenderResult>;

  /**
   * 批量渲染文件树
   *
   * @param templateId - 模板 ID（后端引擎按它读取模板文件与 conditions.yml；WASM 引擎忽略此参数，以传入的 files 为准）
   * @param files - 模板文件列表
   * @param variables - 模板变量对象
   * @returns 每个文件的渲染结果（按顺序对应）
   */
  renderTree(
    templateId: number,
    files: TemplateFile[],
    variables: Record<string, unknown>
  ): Promise<RenderResult[]>;

  /**
   * 获取引擎详细信息
   *
   * @returns 引擎信息，如果不可用则返回 null
   */
  getInfo(): Promise<EngineInfo | null>;
}

/**
 * 渲染结果
 */
export interface RenderResult {
  /** 渲染后的内容（成功时） */
  content: string;
  /** 是否渲染成功 */
  success: boolean;
  /** 错误信息（失败时） */
  error?: RenderError;
  /** 渲染耗时（毫秒） */
  duration?: number;
  /** 渲染的文件路径（批量渲染时） */
  path?: string;
}

/**
 * 渲染错误详情
 */
export interface RenderError {
  /** 错误类型 */
  type: RenderErrorType;
  /** 错误消息 */
  message: string;
  /** 错误所在行号（可选） */
  line?: number;
  /** 错误所在列号（可选） */
  column?: number;
  /** 错误上下文代码片段（可选） */
  context?: string;
  /** 修复建议（可选） */
  suggestion?: string;
}

/**
 * 渲染错误类型枚举
 */
export type RenderErrorType = 'syntax' | 'runtime' | 'network' | 'not_found' | 'unknown';

/**
 * 文件生成条件（与后端 template_core::Condition 的 serde 格式一致）
 * type=if 时用 variable/operator/value；and/or 用 conditions 嵌套；not 用单元素 conditions；switch 用 cases
 */
export interface FileGenCondition {
  type: 'if' | 'and' | 'or' | 'not' | 'switch';
  variable?: string;
  operator?: 'eq' | 'ne' | 'gt' | 'lt' | 'gte' | 'lte' | 'in' | 'notin' | 'contains';
  value?: unknown;
  conditions?: FileGenCondition[];
  cases?: { value: unknown; description?: string }[];
  description?: string;
}

/**
 * 模板文件定义
 */
export interface TemplateFile {
  /** 文件相对路径（使用 Unix 风格 /） */
  path: string;
  /** 文件内容 */
  content: string;
  /** 文件类型 */
  type: TemplateFileType;
  /** 渲染条件表达式（可选） */
  condition?: string;
  /** 文件生成条件（可选，结构与后端 template_core::Condition 一致；条件不满足时该文件及其子树不参与渲染） */
  generateCondition?: FileGenCondition;
  /** 依赖的其他文件路径列表（可选） */
  dependencies?: string[];
  /** 是否为二进制文件 */
  isBinary?: boolean;
}

/**
 * 模板文件类型
 */
export type TemplateFileType = 'template' | 'static' | 'binary';

/**
 * WASM 引擎信息
 *
 * 从后端 API 获取的引擎元数据
 */
export interface EngineInfo {
  /** 引擎版本号 */
  version: string;
  /** 构建时间（ISO 8601 格式） */
  buildTime: string;
  /** WASM 文件大小（字节） */
  size: number;
  /** 支持的过滤器列表 */
  filters: string[];
  /** 支持的内置函数列表 */
  functions: string[];
  /** SHA256 校验和（可选，用于完整性验证） */
  checksum?: string;
}

/**
 * 引擎配置选项
 */
export interface EngineConfig {
  /**
   * 首选引擎类型
   * - 'auto': 自动选择（优先 WASM，失败则回退后端）
   * - 'wasm': 强制使用 WASM（离线模式）
   * - 'backend': 强制使用后端
   */
  preferredEngine: EnginePreference;
  /** WASM 引擎下载 URL（默认使用内置 API） */
  wasmUrl?: string;
  /** 是否启用 IndexedDB 缓存 */
  enableCache: boolean;
  /** 缓存最大有效期（毫秒），默认 7 天 */
  cacheMaxAge: number;
  /** 网络请求超时时间（毫秒） */
  timeout?: number;
}

/**
 * 引擎偏好类型
 */
export type EnginePreference = 'auto' | 'wasm' | 'backend';

/**
 * 引擎状态
 */
export interface EngineState {
  /** 引擎实例 */
  engine: RenderEngine;
  /** 是否可用 */
  available: boolean;
  /** 错误信息（不可用时） */
  error?: string;
  /** 最后检查时间 */
  lastChecked?: number;
}

/**
 * IndexedDB 存储的引擎数据
 */
export interface StoredEngine {
  /** 引擎版本 */
  version: string;
  /** 构建时间 */
  buildTime: string;
  /** WASM 二进制数据 */
  data: ArrayBuffer;
  /** 存储时间戳 */
  storedAt: number;
  /** 校验和 */
  checksum?: string;
}

/**
 * 渲染统计信息
 */
export interface RenderStats {
  /** 当前使用的引擎 */
  engine: string;
  /** 渲染耗时（毫秒） */
  duration: number;
  /** 渲染的文件数量 */
  fileCount?: number;
  /** 是否命中缓存 */
  cacheHit?: boolean;
}

/**
 * 引擎管理器事件
 */
export type EngineManagerEvent =
  | { type: 'engine:initialized'; engine: string }
  | { type: 'engine:switched'; from: string; to: string }
  | { type: 'engine:error'; engine: string; error: string }
  | { type: 'wasm:downloaded'; version: string; size: number }
  | { type: 'wasm:cached'; version: string }
  | { type: 'wasm:loaded'; source: 'cache' | 'network' };

/**
 * 引擎管理器事件监听器
 */
export type EngineManagerEventListener = (event: EngineManagerEvent) => void;

/**
 * 默认引擎配置
 */
export const DEFAULT_ENGINE_CONFIG: EngineConfig = {
  preferredEngine: 'auto',
  enableCache: true,
  cacheMaxAge: 7 * 24 * 60 * 60 * 1000, // 7 天
  timeout: 30000, // 30 秒
};
