<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
      <!-- 自定义标题栏 -->
      <TitleBar :is-dark="isDark" />

      <!-- 主内容区 -->
      <div class="app-layout">
        <div class="app-content">
          <slot></slot>
        </div>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { NConfigProvider, NMessageProvider, darkTheme } from 'naive-ui'
import TitleBar from './TitleBar.vue'

// 暗色主题始终开启
const isDark = ref(true)

// 主题
const theme = computed(() => (isDark.value ? darkTheme : null))

// 监听主题变化，同步到 body 的 class
watch(
  isDark,
  (dark) => {
    if (dark) {
      document.body.classList.add('dark')
    } else {
      document.body.classList.remove('dark')
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background-color: #101014;
  padding-top: 32px; /* 标题栏高度 */
}

.app-content {
  flex: 1;
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
  overflow-y: auto;
  background-color: #101014;
}
</style>

