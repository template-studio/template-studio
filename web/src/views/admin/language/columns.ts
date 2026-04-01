import { h } from 'vue';
import { NTag } from 'naive-ui';
import { BasicColumn } from '@/components/Table';

export interface LanguageData {
  id: number;
  name: string;
  displayName: string;
  display_name: string;
  code: string;
  sort: number;
  isPopular: number;
  is_popular: number;
  created_at: string;
}

export const columns: BasicColumn<LanguageData>[] = [
  {
    title: 'ID',
    key: 'id',
    width: 80,
  },
  {
    title: '语言名称',
    key: 'name',
    width: 150,
    render(row) {
      return h('span', { class: 'language-name' }, row.name);
    },
  },
  {
    title: '显示名称',
    key: 'displayName',
    width: 150,
    render(row) {
      return h(
        'span',
        { class: 'language-display-name' },
        row.displayName || row.display_name || row.name
      );
    },
  },
  {
    title: '语言代码',
    key: 'code',
    width: 100,
    render(row) {
      return h(
        NTag,
        {
          type: 'info',
          size: 'small',
          style: { fontFamily: 'monospace' },
        },
        { default: () => row.code }
      );
    },
  },
  {
    title: '排序',
    key: 'sort',
    width: 100,
    sorter: (a, b) => (a.sort || 0) - (b.sort || 0),
  },
  {
    title: '创建时间',
    key: 'created_at',
    width: 180,
    render(row) {
      return formatDate(row.created_at);
    },
  },
];

function formatDate(dateString) {
  if (!dateString) return '-';
  const date = new Date(dateString);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}
