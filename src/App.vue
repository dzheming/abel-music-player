<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlayerStore } from './stores/player'
import TitleBar from './components/Layout/TitleBar.vue'
import Sidebar from './components/Layout/Sidebar.vue'
import PlayerBar from './components/Layout/PlayerBar.vue'
import PlayerView from './views/PlayerView.vue'
import PlayerRecordView from './views/PlayerRecordView.vue'
import PlayerTimeView from './views/PlayerTimeView.vue'
import PlayerLyricsView from './views/PlayerLyricsView.vue'

const playerStore = usePlayerStore()
const showPlayerView = ref(false)

const playerViewComponent = computed(() => {
    switch (playerStore.playerViewStyle) {
        case 'record': return PlayerRecordView
        case 'time': return PlayerTimeView
        case 'lyrics': return PlayerLyricsView
        default: return PlayerView
    }
})

function openPlayerView() {
    showPlayerView.value = true
}

function closePlayerView() {
    showPlayerView.value = false
    window.dispatchEvent(new Event('player-view-closed'))
}

let saveTimer: ReturnType<typeof setTimeout> | null = null

function debouncedSaveWindow() {
    if (saveTimer) clearTimeout(saveTimer)
    saveTimer = setTimeout(() => {
        invoke('save_window_state').catch(() => {})
    }, 1000)
}

onMounted(async () => {
    await invoke('restore_window_state').catch(() => {})
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const win = getCurrentWindow()
    await win.show()
    await win.setFocus()
    window.addEventListener('resize', debouncedSaveWindow)
})

onUnmounted(() => {
    window.removeEventListener('resize', debouncedSaveWindow)
    if (saveTimer) clearTimeout(saveTimer)
})
</script>

<template>
    <div class="app-layout">
        <TitleBar class="app-title-bar" />
        <Sidebar class="app-sidebar" />
        <main class="app-main">
            <router-view />
        </main>
        <PlayerBar class="app-player-bar" @open-player="openPlayerView"/>
    </div>

    <component :is="playerViewComponent" v-if="showPlayerView" @close="closePlayerView" />
</template>

<style scoped>
.app-layout {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    grid-template-rows: 32px 1fr var(--player-bar-height);
    height: 100vh;
    overflow: hidden;
}

.app-title-bar {
    grid-row: 1;
    grid-column: 1 / -1;
}

.app-sidebar {
    grid-row: 2;
    grid-column: 1;
}

.app-main {
    grid-row: 2;
    grid-column: 2;
    overflow-y: auto;
    background-color: var(--color-bg-primary);
    transition: background-color var(--transition-normal);
}

.app-player-bar {
    grid-row: 3;
    grid-column: 1 / -1;
}
</style>
