import { h } from 'vue';
import { Tag } from 'ant-design-vue';
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
    dataIndex: 'id',
    key: 'id',
    width: 80,
  },
  {
    title: '语言名称',
    dataIndex: 'name',
    key: 'name',
    width: 150,
    customRender({ record }) {
      return h('span', { class: 'language-name' }, record.name);
    },
  },
  {
    title: '显示名称',
    dataIndex: 'displayName',
    key: 'displayName',
    width: 150,
    customRender({ record }) {
      return h(
        'span',
        { class: 'language-display-name' },
        record.displayName || record.display_name || record.name
      );
    },
  },
  {
    title: '语言代码',
    dataIndex: 'code',
    key: 'code',
    width: 100,
    customRender({ record }) {
      return h(
        Tag,
        {
          color: 'blue',
          style: { fontFamily: 'monospace' },
        },
        { default: () => record.code }
      );
    },
  },
  {
    title: '排序',
    dataIndex: 'sort',
    key: 'sort',
    width: 100,
    sorter: (a, b) => (a.sort || 0) - (b.sort || 0),
  },
  {
    title: '创建时间',
    dataIndex: 'created_at',
    key: 'created_at',
    width: 180,
    customRender({ record }) {
      return formatDate(record.created_at);
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
