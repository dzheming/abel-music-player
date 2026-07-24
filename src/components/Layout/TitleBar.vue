<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../../stores/settings'

const appWindow = getCurrentWindow()
const settingsStore = useSettingsStore()

const MAXIMIZE_SAVE_DELAY = 300

async function minimize() {
    await appWindow.minimize()
}

async function toggleMaximize() {
    await appWindow.toggleMaximize()
    setTimeout(() => invoke('save_window_state').catch(() => {}), MAXIMIZE_SAVE_DELAY);
}

async function close() {
    await invoke('save_window_state').catch(() => {})
    if (settingsStore.closeToTray) {
        await appWindow.hide()
    } else {
        await appWindow.close()
    }
}
</script>

<template>
    <div class="title-bar" data-tauri-drag-region>
        <span class="title-text" data-tauri-drag-region>Abel Music Player</span>
        <div class="window-controls">
            <button class="win-btn" @click="minimize" title="最小化">&#x2014;</button>
            <button class="win-btn" @click="toggleMaximize" title="最大化">&#9744;</button>
            <button class="win-btn win-close" @click="close" title="关闭">&#10005;</button>
        </div>
    </div>
</template>

<style scoped>
.title-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    padding-left: 12px;
    background-color: var(--color-bg-sidebar);
    border-bottom: 1px solid var(--color-border);
    user-select: none;
    -webkit-user-select: none;
    transition: background-color var(--transition-normal);
}

.title-text {
    font-size: 12px;
    color: var(--color-text-secondary);
}

.window-controls {
    display: flex;
    height: 100%;
}

.win-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    font-size: 12px;
    color: var(--color-text-secondary);
    border-radius: 0;
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.win-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
}

.win-close:hover {
    background-color: #e81123;
    color: #fff;
}
</style>