/**
 * @description: 弹窗对外暴露的方法
 */
export interface ModalMethods {
  setProps: (props) => void;
  openModal: () => void;
  closeModal: () => void;
  setSubLoading: (status) => void;
}

/**
 * 支持修改的 Modal 属性 (基于 Ant Design Vue Modal)
 */
export interface ModalProps {
  title?: string;
  width?: number | string;
  open?: boolean;
  maskClosable?: boolean;
  okText?: string;
  cancelText?: string;
  centered?: boolean;
  destroyOnClose?: boolean;
  footer?: any;
  zIndex?: number;
  keyboard?: boolean;
  confirmLoading?: boolean;
  closable?: boolean;
  mask?: boolean;
  wrapClassName?: string;
  bodyStyle?: Record<string, string>;
  okType?: string;
  okButtonProps?: object;
  cancelButtonProps?: object;
  // 自定义属性
  subBtuText?: string;
  showIcon?: boolean;
  preset?: string;
  [key: string]: any;
}

export type RegisterFn = (ModalInstance: ModalMethods) => void;

export type UseModalReturnType = [RegisterFn, ModalMethods];
