<template>
  <!-- 变量符号:{ } 大括号 + 中心值点,直接对应模板 {{ 变量 }} 语法;线形风格,currentColor -->
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    style="display: inline-block; vertical-align: -0.125em; flex: none;"
  >
    <!-- 左大括号 -->
    <path
      d="M8.5 3.5H8A2.5 2.5 0 0 0 5.5 6v3.5A1.5 1.5 0 0 1 4 11a1.5 1.5 0 0 1 1.5 1.5V18a2.5 2.5 0 0 0 2.5 2.5h.5"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <!-- 右大括号 -->
    <path
      d="M15.5 3.5H16a2.5 2.5 0 0 1 2.5 2.5v3.5a1.5 1.5 0 0 0 1.5 1.5 1.5 1.5 0 0 0-1.5 1.5V18a2.5 2.5 0 0 1-2.5 2.5h-.5"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <!-- 中心值点 -->
    <circle cx="12" cy="12" r="1.2" fill="currentColor" />
  </svg>
</template>

<script setup>
// size 为高度(px);方形符号;颜色跟随文字色(currentColor)
defineProps({
  size: { type: Number, default: 18 },
})
</script>
