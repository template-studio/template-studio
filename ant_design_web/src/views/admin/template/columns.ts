import { h } from 'vue';
import { Tag } from 'ant-design-vue';
import { BasicColumn } from '@/components/Table';
import { Star } from '@/icons/ionicons5';

export interface TemplateData {
  id: number;
  name: string;
  description: string;
  categoryId: number;
  category_id: number;
  languages: Array<{
    languageId: number;
    isPrimary: number;
    is_primary: number;
  }>;
  createdAt: string;
  created_at: string;
  isFeatured: number;
  is_featured: number;
}

// 外部依赖的函数（在 index.vue 中实现并传入）
let getCategoryName: (categoryId: number) => string = () => null;
let getLanguageName: (languageId: number) => string = () => '';

export const setColumnHelpers = (
  categoryHelper: (categoryId: number) => string,
  languageHelper: (languageId: number) => string
) => {
  getCategoryName = categoryHelper;
  getLanguageName = languageHelper;
};

export const columns: BasicColumn<TemplateData>[] = [
  {
    title: 'ID',
    dataIndex: 'id',
    key: 'id',
    width: 160,
    ellipsis: {
      tooltip: true,
    },
  },
  {
    title: '模板名称',
    dataIndex: 'name',
    key: 'name',
    width: 220,
    customRender({ record }) {
      try {
        return h('div', { style: 'display: flex; align-items: center; gap: 8px' }, [
          record.isFeatured || record.is_featured
            ? h(Star, { style: 'color: #f0a020; font-size: 16px' })
            : null,
          h('span', { class: 'template-name' }, record.name || '未命名模板'),
        ]);
      } catch (error) {
        console.error('模板名称渲染错误:', error);
        return h('span', record.name || '未命名模板');
      }
    },
  },
  {
    title: '描述',
    dataIndex: 'description',
    key: 'description',
    width: 250,
    ellipsis: {
      tooltip: true,
    },
    customRender({ record }) {
      try {
        return record.description || h('span', { class: 'text-placeholder' }, '暂无描述');
      } catch (error) {
        console.error('描述渲染错误:', error);
        return '-';
      }
    },
  },
  {
    title: '分类',
    dataIndex: 'categoryId',
    key: 'categoryId',
    width: 100,
    customRender({ record }) {
      try {
        const categoryId = record.categoryId || record.category_id;
        if (!categoryId) return '-';
        const categoryName = getCategoryName?.(Number(categoryId));
        return categoryName
          ? h(Tag, { color: 'blue' }, { default: () => categoryName })
          : `分类${categoryId}`;
      } catch (error) {
        console.error('分类渲染错误:', error);
        return '-';
      }
    },
  },
  {
    title: '语言',
    dataIndex: 'languages',
    key: 'languages',
    width: 150,
    customRender({ record }) {
      try {
        if (!record.languages || record.languages.length === 0) return '-';
        return h(
          'div',
          { style: 'display: flex; flex-wrap: wrap; gap: 4px' },
          record.languages
            .slice(0, 2)
            .map((lang) => {
              const isPrimary = lang.isPrimary === 1 || lang.is_primary === 1;
              const langName = getLanguageName?.(lang.languageId) || `语言${lang.languageId}`;
              return h(
                Tag,
                {
                  color: isPrimary ? 'blue' : 'default',
                },
                {
                  default: () => langName,
                }
              );
            })
            .concat(
              record.languages.length > 2
                ? [
                    h(
                      'span',
                      { style: 'color: #999; font-size: 12px' },
                      `+${record.languages.length - 2}`
                    ),
                  ]
                : []
            )
        );
      } catch (error) {
        console.error('语言渲染错误:', error);
        return '-';
      }
    },
  },
  {
    title: '创建时间',
    dataIndex: 'createdAt',
    key: 'createdAt',
    width: 180,
    customRender({ record }) {
      try {
        return formatDate(record.createdAt || record.created_at);
      } catch (error) {
        console.error('时间渲染错误:', error);
        return '-';
      }
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
