import { ComponentType } from './types/index';

/**
 * @description: 生成placeholder
 */
export function createPlaceholderMessage(component: ComponentType) {
  if (component === 'Input') return '请输入';
  if (
    ['Select', 'Checkbox', 'Radio', 'Switch', 'DatePicker', 'TimePicker'].includes(
      component
    )
  )
    return '请选择';
  return '';
}

const DATE_TYPE = ['DatePicker', 'DatePicker.MonthPicker', 'DatePicker.WeekPicker', 'TimePicker'];

function genType() {
  return [...DATE_TYPE, 'DatePicker.RangePicker'];
}

/**
 * 时间字段
 */
export const dateItemType = genType();

export function defaultType(component) {
  if (component === 'Input') return '';
  if (component === 'InputNumber') return null;
  return [
    'Select',
    'Checkbox',
    'Radio',
    'Switch',
    'DatePicker',
    'TimePicker',
  ].includes(component)
    ? ''
    : undefined;
}
