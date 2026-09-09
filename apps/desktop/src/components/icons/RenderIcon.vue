<template>
  <!-- 渲染符号:文档 + 代码括号 </>(呼应 logo 母题);线形风格,currentColor -->
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    style="display: inline-block; vertical-align: -0.125em; flex: none;"
  >
    <!-- 文档轮廓(折角) -->
    <path
      d="M14.5 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7.5L14.5 3z"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M14 3.2V7a1 1 0 0 0 1 1h3.6"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <!-- 内部代码括号 < / > -->
    <path
      d="M10.6 12.1 9 14l1.6 1.9"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M13.4 12.1 15 14l-1.6 1.9"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <path
      d="M12.7 11.2l-1.3 5.6"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
    />
  </svg>
</template>

<script setup>
// size 为高度(px);方形符号;颜色跟随文字色(currentColor)
defineProps({
  size: { type: Number, default: 18 },
})
</script>
