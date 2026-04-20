<template>
  <slot name="default"></slot>
</template>

<script lang="ts">
  import { defineComponent } from 'vue';

  export default defineComponent({
    name: 'Application',
    setup() {
      return {};
    },
  });
</script>
