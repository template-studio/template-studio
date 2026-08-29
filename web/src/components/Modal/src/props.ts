import type { PropType } from 'vue';

export const basicProps = {
  // 确认按钮文字
  subBtuText: {
    type: String,
    default: '确认',
  },
  showIcon: {
    type: Boolean,
    default: false,
  },
  width: {
    type: [Number, String] as PropType<number | string>,
    default: 446,
  },
  title: {
    type: String,
    default: '',
  },
  maskClosable: {
    type: Boolean,
    default: false,
  },
  preset: {
    type: String,
    default: 'dialog',
  },
  // Ant Design Vue Modal props
  open: {
    type: Boolean,
    default: false,
  },
  okText: {
    type: String,
    default: undefined,
  },
  cancelText: {
    type: String,
    default: undefined,
  },
  centered: {
    type: Boolean,
    default: false,
  },
  destroyOnClose: {
    type: Boolean,
    default: false,
  },
  footer: {
    type: [String, Object, null] as PropType<string | object | null>,
    default: undefined,
  },
  zIndex: {
    type: Number,
    default: 1000,
  },
  keyboard: {
    type: Boolean,
    default: true,
  },
  confirmLoading: {
    type: Boolean,
    default: false,
  },
  closable: {
    type: Boolean,
    default: true,
  },
  mask: {
    type: Boolean,
    default: true,
  },
  wrapClassName: {
    type: String,
    default: undefined,
  },
  bodyStyle: {
    type: Object as PropType<Record<string, string>>,
    default: undefined,
  },
  okType: {
    type: String,
    default: 'primary',
  },
  okButtonProps: {
    type: Object,
    default: undefined,
  },
  cancelButtonProps: {
    type: Object,
    default: undefined,
  },
};
