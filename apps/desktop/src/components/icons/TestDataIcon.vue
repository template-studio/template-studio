<template>
  <!-- 测试数据符号:锥形烧杯(实验/测试语义,液面线);线形,currentColor -->
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    style="display: inline-block; vertical-align: -0.125em; flex: none;"
  >
    <!-- 烧杯轮廓 -->
    <path
      d="M9.8 2.8v6.6L4.6 19a1.6 1.6 0 0 0 1.4 2.4h12a1.6 1.6 0 0 0 1.4-2.4l-5.2-9.6V2.8"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <!-- 杯口 -->
    <path
      d="M8.2 2.8h7.6"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
    />
    <!-- 液面 -->
    <path
      d="M7.4 15.4h9.2"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
    />
  </svg>
</template>

<script setup>
// size 为高度(px);方形符号;颜色跟随文字色(currentColor)
defineProps({
  size: { type: Number, default: 14 },
})
</script>
