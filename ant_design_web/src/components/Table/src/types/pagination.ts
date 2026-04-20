export interface PaginationProps {
  current?: number; //受控模式下的当前页 (兼容 page)
  page?: number; //受控模式下的当前页 (内部使用，映射到 current)
  total?: number; //总条数 (兼容 itemCount)
  itemCount?: number; //总条数 (内部使用，映射到 total)
  pageCount?: number; //总页数
  pageSize?: number; //受控模式下的分页大小
  pageSizeOptions?: number[]; //每页条数， 可自定义 (兼容 pageSizes)
  pageSizes?: number[]; //每页条数 (内部使用，映射到 pageSizeOptions)
  showSizeChanger?: boolean; //是否显示每页条数的选择器 (兼容 showSizePicker)
  showSizePicker?: boolean; //是否显示每页条数的选择器 (内部使用)
  showQuickJumper?: boolean; //是否显示快速跳转
  showTotal?: ((total: number, range: [number, number]) => string) | boolean; //分页前缀 (兼容 prefix)
  prefix?: any; //分页前缀 (内部使用)
}
