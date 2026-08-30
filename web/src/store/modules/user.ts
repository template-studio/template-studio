import { defineStore } from 'pinia';
import { store } from '@/store';
import { ACCESS_TOKEN, CURRENT_USER } from '@/store/mutation-types';
import { ResultEnum } from '@/enums/httpEnum';

import { getUserInfo as getUserInfoApi, login } from '@/api/system/user';
import { storage } from '@/utils/Storage';
import { useAsyncRoute } from '@/store/modules/asyncRoute';

export type UserInfoType = {
  username: string;
  email: string;
};

export interface IUserState {
  token: string;
  username: string;
  welcome: string;
  avatar: string;
  permissions: any[];
  roles: string[];
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
    roles: [],
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
    getRoles(): string[] {
      return this.roles;
    },
    isAdmin(): boolean {
      return this.roles.some((r) => ['super_admin', 'admin'].includes(r));
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
    setRoles(roles: string[]) {
      this.roles = roles;
    },
    setUserInfo(info: UserInfoType) {
      this.info = info;
    },
    // 登录
    async login(params: any) {
      const response: any = await login(params);
      // 统一信封：{code:0, data}
      const { data: payload, code } = response;
      if (code === ResultEnum.SUCCESS) {
        const ex = 7 * 24 * 60 * 60;
        storage.set(ACCESS_TOKEN, payload.token, ex);
        this.setToken(payload.token);
        if (payload.roles) {
          this.setRoles(payload.roles);
        }
      }
      return response;
    },
    // 获取用户信息
    async getInfo() {
      const data: any = await getUserInfoApi();
      // 统一信封：{code:0, data}
      const payload = data.data;
      if (payload.permissions && payload.permissions.length) {
        const permissionsList = payload.permissions;
        this.setPermissions(permissionsList);

        if (payload.roles) {
          this.setRoles(payload.roles);
        }

        const userInfo = {
          username: payload.username,
          email: payload.email,
          avatar: payload.avatar,
        };
        this.setUserInfo(userInfo);
        this.setAvatar(payload.avatar);
        this.username = payload.username;

        const ex = 7 * 24 * 60 * 60;
        storage.set(CURRENT_USER, { ...userInfo, permissions: permissionsList, roles: payload.roles || [] }, ex);
      } else {
        throw new Error('getInfo: permissionsList must be a non-null array !');
      }
      return payload;
    },
    // 登出
    async logout() {
      this.setPermissions([]);
      this.setRoles([]);
      this.setUserInfo({ username: '', email: '' });
      this.setToken('');
      storage.remove(ACCESS_TOKEN);
      storage.remove(CURRENT_USER);
      // 重置动态路由状态，确保下次进入时重新初始化
      const asyncRouteStore = useAsyncRoute();
      asyncRouteStore.setDynamicRouteAdded(false);
    },
  },
});

// Need to be used outside the setup
export function useUser() {
  return useUserStore(store);
}
