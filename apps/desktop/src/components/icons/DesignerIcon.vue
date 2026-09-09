<template>
  <!-- 变量设计器符号:魔杖 + 四角星光(设计与调校,星光与 AI 品牌符号同族);线形,currentColor -->
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
    style="display: inline-block; vertical-align: -0.125em; flex: none;"
  >
    <!-- 魔杖杆 -->
    <path
      d="M4.2 19.8L14.6 9.4"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
    />
    <!-- 杆柄 -->
    <path
      d="M3.4 16.6l4 4"
      stroke="currentColor"
      stroke-width="2.4"
      stroke-linecap="round"
    />
    <!-- 杆尖四角星 -->
    <path
      d="M17.6 2.6l1 2.4 2.4 1-2.4 1-1 2.4-1-2.4-2.4-1 2.4-1z"
      fill="currentColor"
    />
    <!-- 伴星点 -->
    <circle cx="21" cy="10.6" r="1.1" fill="currentColor" />
  </svg>
</template>

<script setup>
// size 为高度(px);方形符号;颜色跟随文字色(currentColor)
defineProps({
  size: { type: Number, default: 14 },
})
</script>
