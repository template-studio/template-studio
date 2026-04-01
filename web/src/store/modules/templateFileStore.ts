import { defineStore } from 'pinia';

export const useTemplateFileStore = defineStore('templateFile', {
  state: () => ({
    currentFileContent: '',
  }),
  actions: {
    setCurrentFileContent(content) {
      this.currentFileContent = content;
    },
  },
});
