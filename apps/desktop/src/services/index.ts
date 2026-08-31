/**
 * Service 层统一导出（桌面版）
 *
 * 编辑器渲染服务。与 web 版差异：本地引擎为 Tauri 原生实现
 * （TauriEngine）而非 WASM；storage/（IndexedDB 引擎缓存）不适用桌面端未移植。
 */

// 类型定义
export * from './types';

// 渲染引擎服务
export * from './render';
