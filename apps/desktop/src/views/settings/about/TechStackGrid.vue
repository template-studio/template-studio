<template>
  <div class="tech-grid">
    <div class="tech-item" v-for="tech in techStack" :key="tech.name">
      <div class="tech-icon" :style="{ color: tech.color }">
        <component :is="tech.icon" />
      </div>
      <div class="tech-info">
        <div class="tech-name">{{ tech.name }}</div>
        <div class="tech-version">{{ tech.version }}</div>
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  techStack: {
    type: Array,
    required: true
  }
})
</script>

<style scoped>
.tech-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.tech-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  background: var(--color-background);
  border-radius: 6px;
  border: 1px solid var(--color-border);
}

.tech-icon {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.tech-name {
  color: var(--color-text);
  font-size: 13px;
  font-weight: 500;
}

.tech-version {
  color: var(--color-text-secondary);
  font-size: 11px;
}
</style>
