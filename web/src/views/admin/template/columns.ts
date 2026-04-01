import { h } from 'vue';
import { NTag, NIcon } from 'naive-ui';
import { BasicColumn } from '@/components/Table';
import { Star } from '@vicons/ionicons5';

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
    key: 'id',
    width: 80,
  },
  {
    title: '模板名称',
    key: 'name',
    width: 220,
    render(row) {
      try {
        return h('div', { style: 'display: flex; align-items: center; gap: 8px' }, [
          row.isFeatured || row.is_featured
            ? h(NIcon, { color: '#f0a020', size: 16 }, { default: () => h(Star) })
            : null,
          h('span', { class: 'template-name' }, row.name || '未命名模板'),
        ]);
      } catch (error) {
        console.error('模板名称渲染错误:', error);
        return h('span', row.name || '未命名模板');
      }
    },
  },
  {
    title: '描述',
    key: 'description',
    width: 250,
    ellipsis: {
      tooltip: true,
    },
    render(row) {
      try {
        return row.description || h('span', { class: 'text-placeholder' }, '暂无描述');
      } catch (error) {
        console.error('描述渲染错误:', error);
        return '-';
      }
    },
  },
  {
    title: '分类',
    key: 'categoryId',
    width: 100,
    render(row) {
      try {
        const categoryId = row.categoryId || row.category_id;
        if (!categoryId) return '-';
        const categoryName = getCategoryName?.(Number(categoryId));
        return categoryName
          ? h(NTag, { type: 'info', size: 'small' }, { default: () => categoryName })
          : `分类${categoryId}`;
      } catch (error) {
        console.error('分类渲染错误:', error);
        return '-';
      }
    },
  },
  {
    title: '语言',
    key: 'languages',
    width: 150,
    render(row) {
      try {
        if (!row.languages || row.languages.length === 0) return '-';
        return h(
          'div',
          { style: 'display: flex; flex-wrap: wrap; gap: 4px' },
          row.languages
            .slice(0, 2)
            .map((lang) => {
              const isPrimary = lang.isPrimary === 1 || lang.is_primary === 1;
              const langName = getLanguageName?.(lang.languageId) || `语言${lang.languageId}`;
              return h(
                NTag,
                {
                  type: isPrimary ? 'info' : 'default',
                  size: 'small',
                },
                {
                  default: () => langName,
                }
              );
            })
            .concat(
              row.languages.length > 2
                ? [
                    h(
                      'span',
                      { style: 'color: #999; font-size: 12px' },
                      `+${row.languages.length - 2}`
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
    key: 'createdAt',
    width: 180,
    render(row) {
      try {
        return formatDate(row.createdAt || row.created_at);
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
