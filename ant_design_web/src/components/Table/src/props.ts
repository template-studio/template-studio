import type { PropType } from 'vue';
import { propTypes } from '@/utils/propTypes';
import { BasicColumn } from './types/table';
export const basicProps = {
  title: {
    type: String,
    default: null,
  },
  titleTooltip: {
    type: String,
    default: null,
  },
  size: {
    type: String as PropType<'large' | 'middle' | 'small'>,
    default: 'middle',
  },
  dataSource: {
    type: [Object],
    default: () => [],
  },
  columns: {
    type: [Array] as PropType<BasicColumn[]>,
    default: () => [],
    required: true,
  },
  beforeRequest: {
    type: Function as PropType<(...arg: any[]) => void | Promise<any>>,
    default: null,
  },
  request: {
    type: Function as PropType<(...arg: any[]) => Promise<any>>,
    default: null,
  },
  afterRequest: {
    type: Function as PropType<(...arg: any[]) => void | Promise<any>>,
    default: null,
  },
  rowKey: {
    type: [String, Function] as PropType<string | ((record) => string)>,
    default: undefined,
  },
  pagination: {
    type: [Object, Boolean],
    default: () => {},
  },
  //废弃
  showPagination: {
    type: [String, Boolean],
    default: 'auto',
  },
  actionColumn: {
    type: Object as PropType<BasicColumn>,
    default: null,
  },
  canResize: propTypes.bool.def(true),
  resizeHeightOffset: propTypes.number.def(0),
  striped: propTypes.bool.def(false),
  // Ant Design Vue Table 特有 props
  bordered: propTypes.bool.def(false),
  showHeader: propTypes.bool.def(true),
  scroll: {
    type: Object as PropType<{ x?: number | string; y?: number | string }>,
    default: undefined,
  },
  indentSize: propTypes.number.def(15),
  rowSelection: {
    type: Object,
    default: undefined,
  },
  expandedRowKeys: {
    type: Array as PropType<string[]>,
    default: undefined,
  },
  defaultExpandAllRows: propTypes.bool.def(false),
  loading: propTypes.bool.def(false),
};
