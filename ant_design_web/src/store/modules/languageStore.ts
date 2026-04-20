import { defineStore } from 'pinia';
import { getPublicLanguages } from '@/api/public';

export const useLanguageStore = defineStore('language', {
  state: () => ({
    languagesList: [],
    total: 0,
    loaded: false,
  }),
  actions: {
    async fetchLanguages(force = false) {
      if (this.loaded && !force) return this.languagesList;
      try {
        const res = await getPublicLanguages({ all: 1 });
        this.languagesList = res.data.data.languagesList || [];
        this.total = res.data.data.languagesList?.length || 0;
        this.loaded = true;
        return this.languagesList;
      } catch (e) {
        this.loaded = false;
        throw e;
      }
    },
    async getLanguages() {
      if (this.languagesList.length > 0) {
        return this.languagesList;
      } else {
        return await this.fetchLanguages();
      }
    },
  },
});
