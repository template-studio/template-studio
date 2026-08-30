<template>
  <div class="tableAction">
    <div class="flex items-center justify-center">
      <template v-for="(action, index) in getActions" :key="`${index}-${action.label}`">
        <a-button v-bind="getButtonProps(action)" @click="action.onClick?.()" class="mx-1">
          <template #icon v-if="action.icon">
            <component :is="action.icon" />
          </template>
          {{ action.label }}
        </a-button>
      </template>
      <a-dropdown
        v-if="dropDownActions && getDropdownList.length"
        :trigger="['hover']"
      >
        <slot name="more"></slot>
        <a-button v-bind="getMoreProps" class="mx-1" v-if="!$slots.more">
          <div class="flex items-center">
            <span>更多</span>
            <DownOutlined class="ml-1" style="font-size: 14px" />
          </div>
        </a-button>
        <template #overlay>
          <a-menu @click="handleDropdownSelect">
            <a-menu-item
              v-for="(item, idx) in getDropdownList"
              :key="item.key || idx"
            >
              {{ item.label }}
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
    </div>
  </div>
</template>

<script lang="ts">
  import { defineComponent, PropType, computed, toRaw } from 'vue';
  import { ActionItem } from '@/components/Table';
  import { usePermission } from '@/hooks/web/usePermission';
  import { isBoolean, isFunction } from '@/utils/is';
  import { DownOutlined } from '@ant-design/icons-vue';

  export default defineComponent({
    name: 'TableAction',
    components: { DownOutlined },
    props: {
      actions: {
        type: Array as PropType<ActionItem[]>,
        default: null,
        required: true,
      },
      dropDownActions: {
        type: Array as PropType<ActionItem[]>,
        default: null,
      },
      style: {
        type: String as PropType<String>,
        default: 'button',
      },
      select: {
        type: Function as PropType<Function>,
        default: () => {},
      },
    },
    setup(props) {
      const { hasPermission } = usePermission();

      const actionType =
        props.style === 'button' ? 'default' : props.style === 'text' ? 'primary' : 'default';
      const actionText =
        props.style === 'button' ? undefined : props.style === 'text' ? true : undefined;

      const getMoreProps = computed(() => {
        return {
          type: actionType,
          size: 'small',
        };
      });

      const getDropdownList = computed(() => {
        return (toRaw(props.dropDownActions) || [])
          .filter((action) => {
            return hasPermission(action.auth as string[]) && isIfShow(action);
          })
          .map((action, index) => {
            const { popConfirm } = action;
            return {
              size: 'small',
              type: actionType,
              ...action,
              key: (action as any).key || index,
              ...popConfirm,
              onConfirm: popConfirm?.confirm,
              onCancel: popConfirm?.cancel,
            };
          });
      });

      function isIfShow(action: ActionItem): boolean {
        const ifShow = action.ifShow;

        let isIfShow = true;

        if (isBoolean(ifShow)) {
          isIfShow = ifShow;
        }
        if (isFunction(ifShow)) {
          isIfShow = ifShow(action);
        }
        return isIfShow;
      }

      const getActions = computed(() => {
        return (toRaw(props.actions) || [])
          .filter((action) => {
            return hasPermission(action.auth as string[]) && isIfShow(action);
          })
          .map((action) => {
            const { popConfirm } = action;
            //需要展示什么风格，自己修改一下参数
            return {
              size: 'small',
              type: actionType,
              ...action,
              ...(popConfirm || {}),
              onConfirm: popConfirm?.confirm,
              onCancel: popConfirm?.cancel,
              enable: !!popConfirm,
            };
          });
      });

      function getButtonProps(action: any) {
        const { icon, label, popConfirm, ifShow, auth, enable, ...rest } = action;
        return rest;
      }

      function handleDropdownSelect(e: any) {
        props.select?.(e.key);
      }

      return {
        getActions,
        getDropdownList,
        getMoreProps,
        getButtonProps,
        handleDropdownSelect,
      };
    },
  });
</script>
