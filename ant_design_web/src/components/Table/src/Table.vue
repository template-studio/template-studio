<template>
  <div class="table-toolbar">
    <!--顶部左侧区域-->
    <div class="flex items-center table-toolbar-left">
      <template v-if="props.title">
        <div class="table-toolbar-left-title">
          {{ props.title }}
          <a-tooltip trigger="hover" v-if="props.titleTooltip">
            <template #title>
              {{ props.titleTooltip }}
            </template>
            <QuestionCircleOutlined class="ml-1 text-gray-400 cursor-pointer" style="font-size: 18px" />
          </a-tooltip>
        </div>
      </template>
      <slot name="tableTitle"></slot>
    </div>

    <div class="flex items-center leading-none table-toolbar-right">
      <!--顶部右侧区域-->
      <slot name="toolbar"></slot>

      <!--斑马纹-->
      <a-tooltip trigger="hover">
        <template #title>
          <span>表格斑马纹</span>
        </template>
        <div class="mr-2 table-toolbar-right-icon">
          <a-switch v-model:checked="isStriped" @change="setStriped" />
        </div>
      </a-tooltip>
      <a-divider type="vertical" />

      <!--刷新-->
      <a-tooltip trigger="hover">
        <template #title>
          <span>刷新</span>
        </template>
        <div class="table-toolbar-right-icon" @click="reload">
          <ReloadOutlined style="font-size: 18px" />
        </div>
      </a-tooltip>

      <!--密度-->
      <a-tooltip trigger="hover">
        <template #title>
          <span>密度</span>
        </template>
        <div class="table-toolbar-right-icon">
          <a-dropdown :trigger="['click']" @select="densitySelect">
            <ColumnHeightOutlined style="font-size: 18px" />
            <template #overlay>
              <a-menu @click="densitySelect" :selectedKeys="[tableSize]">
                <a-menu-item key="small">紧凑</a-menu-item>
                <a-menu-item key="middle">默认</a-menu-item>
                <a-menu-item key="large">宽松</a-menu-item>
              </a-menu>
            </template>
          </a-dropdown>
        </div>
      </a-tooltip>

      <!--表格设置单独抽离成组件-->
      <ColumnSetting />
    </div>
  </div>
  <div class="s-table">
    <a-table
      ref="tableElRef"
      v-bind="getBindValues"
      :striped="isStriped"
      :pagination="pagination"
      @change="handleTableChange"
    >
      <template #[item]="data" v-for="item in Object.keys($slots)" :key="item">
        <slot :name="item" v-bind="data"></slot>
      </template>
    </a-table>
  </div>
</template>

<script lang="ts" setup>
  import { ref, unref, toRaw, computed, onMounted, nextTick } from 'vue';
  import { ReloadOutlined, ColumnHeightOutlined, QuestionCircleOutlined } from '@ant-design/icons-vue';
  import { createTableContext } from './hooks/useTableContext';

  import ColumnSetting from './components/settings/ColumnSetting.vue';

  import { useLoading } from './hooks/useLoading';
  import { useColumns } from './hooks/useColumns';
  import { useDataSource } from './hooks/useDataSource';
  import { usePagination } from './hooks/usePagination';

  import { basicProps } from './props';

  import { BasicTableProps } from './types/table';

  import { getViewportOffset } from '@/utils/domUtils';
  import { useWindowSizeFn } from '@/hooks/event/useWindowSizeFn';
  import { isBoolean } from '@/utils/is';

  const emit = defineEmits([
    'fetch-success',
    'fetch-error',
    'update:checked-row-keys',
    'edit-end',
    'edit-cancel',
    'edit-row-end',
    'edit-change',
  ]);

  const props = defineProps({ ...basicProps });
  const deviceHeight = ref(150);
  const tableElRef = ref<ComponentRef>(null);
  const wrapRef = ref<Nullable<HTMLDivElement>>(null);
  let paginationEl: HTMLElement | null;
  const isStriped = ref(props.striped || false);
  const tableData = ref<Recordable[]>([]);
  const innerPropsRef = ref<Partial<BasicTableProps>>();

  const getProps = computed(() => {
    return { ...props, ...unref(innerPropsRef) } as BasicTableProps;
  });

  const tableSize = ref(unref(getProps as any).size || 'middle');

  const { getLoading, setLoading } = useLoading(getProps);

  const { getPaginationInfo, setPagination } = usePagination(getProps);

  const { getDataSourceRef, getDataSource, getRowKey, reload } = useDataSource(
    getProps,
    {
      getPaginationInfo,
      setPagination,
      tableData,
      setLoading,
    },
    emit
  );

  const { getPageColumns, setColumns, getColumns, getCacheColumns, setCacheColumnsField } =
    useColumns(getProps);

  //Ant Design Vue 的 @change 事件，page 和 pageSize 都通过这个事件处理
  function handleTableChange(pag: any) {
    const { current, pageSize } = pag;
    setPagination({ current, pageSize, page: current });
    reload();
  }

  //密度切换
  function densitySelect(e: any) {
    tableSize.value = e.key || e;
  }

  //获取表格大小
  const getTableSize = computed(() => tableSize.value);

  //组装表格信息
  const getBindValues = computed(() => {
    const tableData = unref(getDataSourceRef);
    const maxHeight = tableData.length ? `${unref(deviceHeight)}px` : undefined;
    return {
      ...unref(getProps),
      loading: unref(getLoading),
      columns: toRaw(unref(getPageColumns)),
      rowKey: unref(getRowKey),
      dataSource: tableData,
      size: unref(getTableSize),
      scroll: maxHeight ? { y: maxHeight } : undefined,
      title: undefined, // 重置为空 避免绑定到 table 上面
    };
  });

  //获取分页信息
  const pagination = computed(() => {
    const paginationInfo = toRaw(unref(getPaginationInfo));
    if (isBoolean(paginationInfo)) {
      return paginationInfo;
    }
    // 转换为 Ant Design Vue 的分页格式
    return {
      current: paginationInfo.current || paginationInfo.page || 1,
      pageSize: paginationInfo.pageSize || 10,
      total: paginationInfo.total || paginationInfo.itemCount || 0,
      pageSizeOptions: paginationInfo.pageSizeOptions || paginationInfo.pageSizes,
      showSizeChanger: paginationInfo.showSizeChanger ?? paginationInfo.showSizePicker ?? true,
      showQuickJumper: paginationInfo.showQuickJumper ?? true,
      showTotal: paginationInfo.showTotal || paginationInfo.prefix
        ? (total: number) => `共 ${total} 条`
        : undefined,
    };
  });

  function setProps(props: Partial<BasicTableProps>) {
    innerPropsRef.value = { ...unref(innerPropsRef), ...props };
  }

  const setStriped = (value: boolean) => (isStriped.value = value);

  const tableAction = {
    reload,
    setColumns,
    setLoading,
    setProps,
    getColumns,
    getDataSource,
    getPageColumns,
    getCacheColumns,
    setCacheColumnsField,
    emit,
  };

  const getCanResize = computed(() => {
    const { canResize } = unref(getProps);
    return canResize;
  });

  async function computeTableHeight() {
    const table = unref(tableElRef);
    if (!table) return;
    if (!unref(getCanResize)) return;
    const tableEl: any = table?.$el;
    const headEl = tableEl.querySelector('.ant-table-thead');
    const { bottomIncludeBody } = getViewportOffset(headEl);
    const headerH = 64;
    let paginationH = 2;
    let marginH = 24;
    if (!isBoolean(unref(pagination))) {
      paginationEl = tableEl.querySelector('.ant-table-pagination') as HTMLElement;
      if (paginationEl) {
        const offsetHeight = paginationEl.offsetHeight;
        paginationH += offsetHeight || 0;
      } else {
        paginationH += 28;
      }
    }
    let height =
      bottomIncludeBody - (headerH + paginationH + marginH + (props.resizeHeightOffset || 0));
    const maxHeight = props.maxHeight;
    height = maxHeight && maxHeight < height ? maxHeight : height;
    deviceHeight.value = height;
  }

  useWindowSizeFn(computeTableHeight, 280);

  onMounted(() => {
    nextTick(() => {
      computeTableHeight();
    });
  });

  createTableContext({ ...tableAction, wrapRef, getBindValues });

  defineExpose(tableAction);
</script>
<style lang="less" scoped>
  .table-toolbar {
    display: flex;
    justify-content: space-between;
    padding: 0 0 16px 0;

    &-left {
      display: flex;
      align-items: center;
      justify-content: flex-start;
      flex: 1;

      &-title {
        display: flex;
        align-items: center;
        justify-content: flex-start;
        font-size: 16px;
        font-weight: 600;
      }
    }

    &-right {
      display: flex;
      justify-content: flex-end;
      flex: 1;

      &-icon {
        margin-left: 12px;
        font-size: 16px;
        cursor: pointer;
        color: var(--text-color);

        :hover {
          color: #1890ff;
        }
      }
    }
  }

  .table-toolbar-inner-popover-title {
    padding: 2px 0;
  }
</style>
