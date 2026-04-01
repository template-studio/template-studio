import { defineStore } from 'pinia';
import { store } from '@/store';
import { ACCESS_TOKEN, CURRENT_USER, IS_SCREENLOCKED } from '@/store/mutation-types';
import { ResultEnum } from '@/enums/httpEnum';

import { getUserInfo as getUserInfoApi, login } from '@/api/system/user';
import { storage } from '@/utils/Storage';

export type UserInfoType = {
  // TODO: add your own data
  username: string;
  email: string;
};

export interface IUserState {
  token: string;
  username: string;
  welcome: string;
  avatar: string;
  permissions: any[];
  info: UserInfoType;
}

export const useUserStore = defineStore({
  id: 'app-user',
  state: (): IUserState => ({
    token: storage.get(ACCESS_TOKEN, ''),
    username: '',
    welcome: '',
    avatar: '',
    permissions: [],
    info: storage.get(CURRENT_USER, {}),
  }),
  getters: {
    getToken(): string {
      return this.token;
    },
    getAvatar(): string {
      return this.avatar;
    },
    getNickname(): string {
      return this.username;
    },
    getPermissions(): [any][] {
      return this.permissions;
    },
    getUserInfo(): UserInfoType {
      return this.info;
    },
  },
  actions: {
    setToken(token: string) {
      this.token = token;
    },
    setAvatar(avatar: string) {
      this.avatar = avatar;
    },
    setPermissions(permissions) {
      this.permissions = permissions;
    },
    setUserInfo(info: UserInfoType) {
      this.info = info;
    },
    // 登录
    async login(params: any) {
      // 开发环境下固定登录成功，避免依赖后端
      if (import.meta.env.DEV) {
        const mockResult = {
          token: 'dev-mock-token-' + Date.now(),
          username: params.username || 'admin',
          email: 'admin@example.com',
          avatar: '',
          permissions: [
            { value: 'dashboard', label: '仪表盘' },
            { value: 'template', label: '模板管理' },
            { value: 'category', label: '分类管理' },
            { value: 'language', label: '语言管理' },
          ],
        };

        const ex = 7 * 24 * 60 * 60;
        storage.set(ACCESS_TOKEN, mockResult.token, ex);
        storage.set(CURRENT_USER, mockResult, ex);
        storage.set(IS_SCREENLOCKED, false);
        this.setToken(mockResult.token);
        this.setUserInfo(mockResult);
        this.setPermissions(mockResult.permissions);

        return {
          code: ResultEnum.SUCCESS,
          result: mockResult,
          message: '登录成功（开发模式）',
        };
      }

      // 生产环境调用真实API
      const response = await login(params);
      const { result, code } = response;
      if (code === ResultEnum.SUCCESS) {
        const ex = 7 * 24 * 60 * 60;
        storage.set(ACCESS_TOKEN, result.token, ex);
        storage.set(CURRENT_USER, result, ex);
        storage.set(IS_SCREENLOCKED, false);
        this.setToken(result.token);
        this.setUserInfo(result);
      }
      return response;
    },

    // 获取用户信息
    async getInfo() {
      // 开发环境下返回模拟用户信息
      if (import.meta.env.DEV) {
        const mockResult = {
          username: this.username || 'admin',
          email: 'admin@example.com',
          avatar: '',
          permissions: [
            { value: 'dashboard', label: '仪表盘' },
            { value: 'template', label: '模板管理' },
            { value: 'category', label: '分类管理' },
            { value: 'language', label: '语言管理' },
          ],
        };

        this.setPermissions(mockResult.permissions);
        this.setUserInfo(mockResult);
        this.setAvatar(mockResult.avatar);
        return mockResult;
      }

      // 生产环境调用真实API
      const data = await getUserInfoApi();
      const { result } = data;
      if (result.permissions && result.permissions.length) {
        const permissionsList = result.permissions;
        this.setPermissions(permissionsList);
        this.setUserInfo(result);
      } else {
        throw new Error('getInfo: permissionsList must be a non-null array !');
      }
      this.setAvatar(result.avatar);
      return result;
    },

    // 登出
    async logout() {
      this.setPermissions([]);
      this.setUserInfo({ username: '', email: '' });
      storage.remove(ACCESS_TOKEN);
      storage.remove(CURRENT_USER);
    },
  },
});

// Need to be used outside the setup
export function useUser() {
  return useUserStore(store);
}
