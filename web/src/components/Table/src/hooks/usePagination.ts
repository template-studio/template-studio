import type { PaginationProps } from '../types/pagination';
import type { BasicTableProps } from '../types/table';
import { computed, unref, ref, ComputedRef, watch } from 'vue';

import { isBoolean } from '@/utils/is';
import { DEFAULTPAGESIZE, PAGESIZES } from '../const';

export function usePagination(refProps: ComputedRef<BasicTableProps>) {
  const configRef = ref<PaginationProps>({});
  const show = ref(true);

  watch(
    () => unref(refProps).pagination,
    (pagination) => {
      if (!isBoolean(pagination) && pagination) {
        configRef.value = {
          ...unref(configRef),
          ...(pagination ?? {}),
        };
      }
    }
  );

  const getPaginationInfo = computed((): PaginationProps | boolean => {
    const { pagination } = unref(refProps);
    if (!unref(show) || (isBoolean(pagination) && !pagination)) {
      return false;
    }
    return {
      current: 1, //当前页 (Ant Design Vue)
      page: 1, //当前页 (内部兼容)
      pageSize: DEFAULTPAGESIZE, //分页大小
      pageSizeOptions: PAGESIZES, // 每页条数 (Ant Design Vue)
      pageSizes: PAGESIZES, // 每页条数 (内部兼容)
      showSizeChanger: true, // (Ant Design Vue)
      showSizePicker: true, // (内部兼容)
      showQuickJumper: true,
      showTotal: (total: number) => `共 ${total} 条`, // (Ant Design Vue)
      prefix: (pagingInfo: any) => `共 ${pagingInfo.itemCount} 条`, // (内部兼容)
      ...(isBoolean(pagination) ? {} : pagination),
      ...unref(configRef),
    };
  });

  function setPagination(info: Partial<PaginationProps>) {
    const paginationInfo = unref(getPaginationInfo);
    const newInfo = {
      ...(!isBoolean(paginationInfo) ? paginationInfo : {}),
      ...info,
    };
    // 同步 page/current
    if (info.page !== undefined) {
      newInfo.current = info.page;
    }
    if (info.current !== undefined) {
      newInfo.page = info.current;
    }
    // 同步 itemCount/total
    if (info.itemCount !== undefined) {
      newInfo.total = info.itemCount;
    }
    if (info.total !== undefined) {
      newInfo.itemCount = info.total;
    }
    configRef.value = newInfo;
  }

  function getPagination() {
    return unref(getPaginationInfo);
  }

  function getShowPagination() {
    return unref(show);
  }

  async function setShowPagination(flag: boolean) {
    show.value = flag;
  }

  return { getPagination, getPaginationInfo, setShowPagination, getShowPagination, setPagination };
}
