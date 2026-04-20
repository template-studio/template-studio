import type { PropType } from 'vue';

export const basicProps = {
  // Ant Design Vue Upload props
  action: {
    type: String,
    default: undefined,
  },
  headers: {
    type: Object as PropType<Record<string, string>>,
    default: undefined,
  },
  name: {
    type: String,
    default: 'file',
  },
  data: {
    type: [Object, Function] as PropType<Record<string, any> | ((file: any) => Record<string, any>)>,
    default: undefined,
  },
  multiple: {
    type: Boolean,
    default: false,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  method: {
    type: String as PropType<'POST' | 'PUT' | 'PATCH' | 'post' | 'put' | 'patch'>,
    default: 'POST',
  },
  listType: {
    type: String as PropType<'text' | 'picture' | 'picture-card'>,
    default: 'picture-card',
  },
  showUploadList: {
    type: [Boolean, Object] as PropType<boolean | object>,
    default: false,
  },
  customRequest: {
    type: Function as PropType<(options: any) => void>,
    default: undefined,
  },
  withCredentials: {
    type: Boolean,
    default: false,
  },
  directory: {
    type: Boolean,
    default: false,
  },
  beforeUpload: {
    type: Function as PropType<(file: File, fileList: File[]) => boolean | Promise<boolean>>,
    default: undefined,
  },
  // 自定义属性
  accept: {
    type: String,
    default: '.jpg,.png,.jpeg,.svg,.gif',
  },
  helpText: {
    type: String as PropType<string>,
    default: '',
  },
  maxSize: {
    type: Number as PropType<number>,
    default: 2,
  },
  maxNumber: {
    type: Number as PropType<number>,
    default: Infinity,
  },
  value: {
    type: Array as PropType<string[]>,
    default: () => [],
  },
  width: {
    type: Number as PropType<number>,
    default: 104,
  },
  height: {
    type: Number as PropType<number>,
    default: 104, //建议不小于这个尺寸 太小页面可能显示有异常
  },
};
