import { h } from 'vue';
import { Tag } from 'ant-design-vue';
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
    dataIndex: 'id',
    key: 'id',
    width: 80,
  },
  {
    title: '预设名称',
    dataIndex: 'name',
    key: 'name',
    width: 120,
    customRender({ record }) {
      return h(
        Tag,
        {
          color: 'blue',
        },
        {
          default: () => record.name,
        }
      );
    },
  },
  {
    title: '显示名称',
    dataIndex: 'displayName',
    key: 'displayName',
    width: 150,
    customRender({ record }) {
      return h('span', { class: 'preset-display-name' }, record.displayName || record.name);
    },
  },
  {
    title: '分类',
    dataIndex: 'category',
    key: 'category',
    width: 80,
    customRender({ record }) {
      return h(
        Tag,
        {
          color: record.category === 'system' ? 'blue' : 'green',
        },
        {
          default: () => (record.category === 'system' ? '系统' : '自定义'),
        }
      );
    },
  },
  {
    title: '描述',
    dataIndex: 'description',
    key: 'description',
    width: 180,
    ellipsis: true,
    customRender({ record }) {
      return record.description || h('span', { class: 'text-placeholder' }, '暂无描述');
    },
  },
  {
    title: '版本',
    dataIndex: 'version',
    key: 'version',
    width: 80,
    customRender({ record }) {
      return record.version || '1.0';
    },
  },
  {
    title: '排序',
    dataIndex: 'sort',
    key: 'sort',
    width: 80,
    sorter: (a, b) => (a.sort || 0) - (b.sort || 0),
  },
  {
    title: '状态',
    dataIndex: 'isEnabled',
    key: 'isEnabled',
    width: 80,
    customRender({ record }) {
      return h(
        Tag,
        {
          color: record.isEnabled === 1 ? 'green' : 'red',
        },
        {
          default: () => (record.isEnabled === 1 ? '启用' : '禁用'),
        }
      );
    },
  },
  {
    title: '创建时间',
    dataIndex: 'createdAt',
    key: 'createdAt',
    width: 180,
    customRender({ record }) {
      return formatDate(record.createdAt);
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
