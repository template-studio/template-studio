import { h } from 'vue';
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
    dataIndex: 'id',
    key: 'id',
    width: 80,
  },
  {
    title: '分类名称',
    dataIndex: 'name',
    key: 'name',
    width: 150,
    customRender({ record }) {
      return h('span', { class: 'category-name' }, record.name);
    },
  },
  {
    title: '描述',
    dataIndex: 'description',
    key: 'description',
    width: 200,
    ellipsis: {
      tooltip: true,
    },
    customRender({ record }) {
      return record.description || h('span', { class: 'text-placeholder' }, '暂无描述');
    },
  },
  {
    title: '排序',
    dataIndex: 'sort',
    key: 'sort',
    width: 100,
    sorter: (a, b) => a.sort - b.sort,
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
