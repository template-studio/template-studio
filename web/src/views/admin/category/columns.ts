import { h } from 'vue';
import { NSwitch } from 'naive-ui';
import { BasicColumn } from '@/components/Table';

export interface CategoryData {
  id: number;
  name: string;
  description: string;
  sort: number;
  status: number;
  created_at: string;
}

export const columns: BasicColumn<CategoryData>[] = [
  {
    title: 'ID',
    key: 'id',
    width: 80,
  },
  {
    title: '分类名称',
    key: 'name',
    width: 150,
    render(row) {
      return h('span', { class: 'category-name' }, row.name);
    },
  },
  {
    title: '描述',
    key: 'description',
    width: 200,
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      return row.description || h('span', { class: 'text-placeholder' }, '暂无描述');
    },
  },
  {
    title: '排序',
    key: 'sort',
    width: 100,
    sorter: (a, b) => a.sort - b.sort,
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
