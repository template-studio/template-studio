<template>
  <footer class="footer-bar">
    <div class="footer-main">
      <div class="footer-links">
        <a href="#" @click.prevent>关于我们</a>
        <a href="#" @click.prevent>用户协议</a>
        <a href="#" @click.prevent>隐私政策</a>
        <a href="#" @click.prevent>联系我们</a>
        <a href="#" @click.prevent>友情链接</a>
      </div>
      <div class="footer-info">
        <span>© 2025 Template Studio Lite</span>
        <span class="footer-divider">|</span>
        <span>基于 GoFrame & Vue3 构建</span>
        <span class="footer-divider">|</span>
        <span>Powered by Naive UI & Vite</span>
      </div>
      <div class="footer-contact">
        <span>反馈邮箱：feedback@templateStudio.com</span>
        <span class="footer-divider">|</span>
        <span>技术支持：support@templateStudio.com</span>
      </div>
    </div>
  </footer>
</template>

<script setup></script>

<style scoped>
  .footer-bar {
    position: static;
    width: 100%;
    background: #f8f9fa;
    border-top: 1px solid #ececec;
    z-index: 100;
    padding: 32px 0 18px 0;
    margin-top: 48px;
  }

  .footer-main {
    width: 80vw;
    max-width: 1280px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }

  .footer-links {
    display: flex;
    gap: 24px;
    font-size: 15px;
    color: #666;
    margin-bottom: 4px;
  }

  .footer-links a {
    color: #666;
    text-decoration: none;
    transition: color 0.2s;
  }

  .footer-links a:hover {
    color: #18a058;
  }

  .footer-info {
    color: #888;
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .footer-divider {
    margin: 0 6px;
    color: #ccc;
  }

  .footer-contact {
    color: #aaa;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
</style>
