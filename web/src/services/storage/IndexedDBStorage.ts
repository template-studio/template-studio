/**
 * IndexedDB 存储服务
 *
 * 用于缓存 WASM 引擎二进制文件，支持离线使用
 */

import type { StoredEngine } from '../types';

const DB_NAME = 'TemplateStudio';
const DB_VERSION = 1;
const ENGINE_STORE = 'wasm_engine';
const META_STORE = 'meta';

/**
 * IndexedDB 存储服务类
 *
 * 提供对 WASM 引擎缓存的 CRUD 操作
 */
export class IndexedDBStorage {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  /**
   * 打开数据库连接
   *
   * 使用单例模式，多次调用返回同一个 Promise
   */
  async open(): Promise<void> {
    // 如果已经有初始化中的 Promise，直接返回
    if (this.initPromise) {
      return this.initPromise;
    }

    // 如果已经连接，直接返回
    if (this.db) {
      return Promise.resolve();
    }

    this.initPromise = this.doOpen();
    return this.initPromise;
  }

  private async doOpen(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        const error = request.error;
        console.error('[IndexedDBStorage] Failed to open database:', error);
        this.initPromise = null;
        reject(new Error(`Failed to open IndexedDB: ${error?.message || 'Unknown error'}`));
      };

      request.onsuccess = () => {
        this.db = request.result;
        console.log('[IndexedDBStorage] Database opened successfully');
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        console.log('[IndexedDBStorage] Upgrading database schema...');

        // 创建引擎存储
        if (!db.objectStoreNames.contains(ENGINE_STORE)) {
          db.createObjectStore(ENGINE_STORE);
          console.log(`[IndexedDBStorage] Created object store: ${ENGINE_STORE}`);
        }

        // 创建元数据存储
        if (!db.objectStoreNames.contains(META_STORE)) {
          db.createObjectStore(META_STORE);
          console.log(`[IndexedDBStorage] Created object store: ${META_STORE}`);
        }
      };

      request.onblocked = () => {
        console.warn('[IndexedDBStorage] Database open blocked by another connection');
      };
    });
  }

  /**
   * 确保数据库已连接
   */
  private async ensureOpen(): Promise<void> {
    if (!this.db) {
      await this.open();
    }
  }

  /**
   * 获取缓存的 WASM 引擎
   *
   * @returns 缓存的引擎数据，如果不存在返回 null
   */
  async getEngine(): Promise<StoredEngine | null> {
    await this.ensureOpen();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([ENGINE_STORE], 'readonly');
      const store = transaction.objectStore(ENGINE_STORE);
      const request = store.get('current');

      request.onerror = () => {
        console.error('[IndexedDBStorage] Failed to get engine:', request.error);
        reject(new Error(`Failed to get engine: ${request.error?.message}`));
      };

      request.onsuccess = () => {
        const result = request.result as StoredEngine | undefined;
        resolve(result || null);
      };
    });
  }

  /**
   * 保存 WASM 引擎到缓存
   *
   * @param engine - 要存储的引擎数据
   */
  async setEngine(engine: StoredEngine): Promise<void> {
    await this.ensureOpen();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([ENGINE_STORE], 'readwrite');
      const store = transaction.objectStore(ENGINE_STORE);
      const request = store.put(engine, 'current');

      request.onerror = () => {
        console.error('[IndexedDBStorage] Failed to save engine:', request.error);
        reject(new Error(`Failed to save engine: ${request.error?.message}`));
      };

      request.onsuccess = () => {
        console.log(
          `[IndexedDBStorage] Engine saved: version=${engine.version}, size=${engine.data.byteLength} bytes`
        );
        resolve();
      };

      transaction.oncomplete = () => {
        // 事务完成
      };

      transaction.onerror = () => {
        console.error('[IndexedDBStorage] Transaction failed:', transaction.error);
      };
    });
  }

  /**
   * 检查是否有缓存的引擎
   */
  async hasEngine(): Promise<boolean> {
    const engine = await this.getEngine();
    return engine !== null;
  }

  /**
   * 获取缓存引擎的版本信息
   *
   * @returns 版本字符串，如果不存在返回 null
   */
  async getEngineVersion(): Promise<string | null> {
    const engine = await this.getEngine();
    return engine?.version || null;
  }

  /**
   * 检查缓存的引擎是否过期
   *
   * @param maxAge - 最大有效期（毫秒）
   * @returns true 表示已过期
   */
  async isEngineExpired(maxAge: number): Promise<boolean> {
    const engine = await this.getEngine();
    if (!engine) {
      return true;
    }

    const age = Date.now() - engine.storedAt;
    return age > maxAge;
  }

  /**
   * 清除缓存的引擎
   */
  async clearEngine(): Promise<void> {
    await this.ensureOpen();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([ENGINE_STORE], 'readwrite');
      const store = transaction.objectStore(ENGINE_STORE);
      const request = store.delete('current');

      request.onerror = () => {
        console.error('[IndexedDBStorage] Failed to clear engine:', request.error);
        reject(new Error(`Failed to clear engine: ${request.error?.message}`));
      };

      request.onsuccess = () => {
        console.log('[IndexedDBStorage] Engine cache cleared');
        resolve();
      };
    });
  }

  /**
   * 获取存储的元数据
   *
   * @param key - 元数据键名
   * @returns 元数据值
   */
  async getMeta<T>(key: string): Promise<T | null> {
    await this.ensureOpen();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([META_STORE], 'readonly');
      const store = transaction.objectStore(META_STORE);
      const request = store.get(key);

      request.onerror = () => {
        console.error(`[IndexedDBStorage] Failed to get meta "${key}":`, request.error);
        reject(new Error(`Failed to get meta: ${request.error?.message}`));
      };

      request.onsuccess = () => {
        resolve(request.result ?? null);
      };
    });
  }

  /**
   * 保存元数据
   *
   * @param key - 元数据键名
   * @param value - 元数据值
   */
  async setMeta<T>(key: string, value: T): Promise<void> {
    await this.ensureOpen();

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([META_STORE], 'readwrite');
      const store = transaction.objectStore(META_STORE);
      const request = store.put(value, key);

      request.onerror = () => {
        console.error(`[IndexedDBStorage] Failed to set meta "${key}":`, request.error);
        reject(new Error(`Failed to set meta: ${request.error?.message}`));
      };

      request.onsuccess = () => {
        resolve();
      };
    });
  }

  /**
   * 获取存储使用情况
   *
   * @returns 已使用的存储空间（字节）
   */
  async getStorageUsage(): Promise<number> {
    const engine = await this.getEngine();
    return engine?.data.byteLength || 0;
  }

  /**
   * 关闭数据库连接
   */
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
      this.initPromise = null;
      console.log('[IndexedDBStorage] Database connection closed');
    }
  }

  /**
   * 检查 IndexedDB 是否可用
   */
  static isAvailable(): boolean {
    try {
      return typeof indexedDB !== 'undefined' && indexedDB !== null;
    } catch {
      return false;
    }
  }
}
