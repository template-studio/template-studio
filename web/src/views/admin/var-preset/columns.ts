import { h } from 'vue';
import { NTag } from 'naive-ui';
import { BasicColumn } from '@/components/Table';

export interface VarPresetData {
  id: number;
  name: string;
  displayName: string;
  description: string;
  category: string;
  icon: string;
  sort: number;
  version: string;
  isEnabled: number;
  createdAt: string;
}

export const columns: BasicColumn<VarPresetData>[] = [
  {
    title: 'ID',
    key: 'id',
    width: 80,
  },
  {
    title: '预设名称',
    key: 'name',
    width: 120,
    render(row) {
      return h(
        NTag,
        {
          type: 'info',
          size: 'medium',
        },
        {
          default: () => row.name,
        }
      );
    },
  },
  {
    title: '显示名称',
    key: 'displayName',
    width: 150,
    render(row) {
      return h('span', { class: 'preset-display-name' }, row.displayName || row.name);
    },
  },
  {
    title: '分类',
    key: 'category',
    width: 80,
    render(row) {
      return h(
        NTag,
        {
          type: row.category === 'system' ? 'info' : 'success',
          size: 'small',
        },
        {
          default: () => (row.category === 'system' ? '系统' : '自定义'),
        }
      );
    },
  },
  {
    title: '描述',
    key: 'description',
    width: 180,
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      return row.description || h('span', { class: 'text-placeholder' }, '暂无描述');
    },
  },
  {
    title: '版本',
    key: 'version',
    width: 80,
    render(row) {
      return row.version || '1.0';
    },
  },
  {
    title: '排序',
    key: 'sort',
    width: 80,
    sorter: (a, b) => (a.sort || 0) - (b.sort || 0),
  },
  {
    title: '状态',
    key: 'isEnabled',
    width: 80,
    render(row) {
      return h(
        NTag,
        {
          type: row.isEnabled === 1 ? 'success' : 'error',
          size: 'small',
        },
        {
          default: () => (row.isEnabled === 1 ? '启用' : '禁用'),
        }
      );
    },
  },
  {
    title: '创建时间',
    key: 'createdAt',
    width: 180,
    render(row) {
      return formatDate(row.createdAt);
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
