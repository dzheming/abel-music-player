<script setup lang="ts">
import { ref } from 'vue'
import Equalizer from './Equalizer.vue'
import AudioEffects from './AudioEffects.vue'

const activeTab = ref<'eq' | 'effects'>('eq')
</script>

<template>
    <div class="effects-panel">
        <div class="panel-tabs">
            <button
                class="panel-tab"
                :class="{ active: activeTab === 'eq' }"
                @click="activeTab = 'eq'"
            >均衡器</button>
            <button
                class="panel-tab"
                :class="{ active: activeTab === 'effects' }"
                @click="activeTab = 'effects'"
            >音效</button>
        </div>
        <div class="panel-content">
            <Equalizer v-if="activeTab === 'eq'" />
            <AudioEffects v-else />
        </div>
    </div>
</template>

<style scoped>
.effects-panel {
    min-width: 320px;
}

.panel-content {
    height: 200px;
}

.panel-tabs {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
}

.panel-tab {
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-tertiary);
    border-bottom: 2px solid transparent;
    transition: all var(--transition-fast);
}

.panel-tab:hover {
    color: var(--color-text-primary);
}

.panel-tab.active {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
}
</style>