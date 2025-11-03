<script setup>
import { ref } from 'vue'
import { NTabs, NTabPane } from 'naive-ui'
import AppLayout from './components/layout/AppLayout.vue'
import TokenManager from './components/TokenManager.vue'
import TokenParser from './components/TokenParser.vue'

const activeTab = ref('token-manager')


</script>

<template>
  <AppLayout>
    <div style="height: 100%; display: flex; flex-direction: column; overflow: hidden;">
      <NTabs
        v-model:value="activeTab"
        type="line"
        animated
        style="flex: 1; display: flex; flex-direction: column; overflow: hidden;"
      >
        <NTabPane name="token-manager" tab="Token 管理" pane-style="height: 100%; overflow: hidden;" style="flex: 1; height: 100%; overflow: hidden;">
          <TokenManager />
        </NTabPane>
        <NTabPane name="token-parser" tab="Token 解析" pane-style="height: 100%; overflow: hidden;" style="flex: 1; height: 100%; overflow: hidden;">
          <TokenParser />
        </NTabPane>
      </NTabs>
    </div>
  </AppLayout>
</template>

<style scoped>
:deep(.n-tabs) {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

:deep(.n-tabs-nav) {
  padding: 0 20px;
  flex-shrink: 0;
}

:deep(.n-tabs-pane-wrapper) {
  flex: 1;
  overflow: hidden;
}

:deep(.n-tab-pane) {
  height: 100%;
  overflow: auto;
}
</style>
