<script setup lang="ts">
import { ref } from 'vue'
import { useSettingsStore } from '../stores/settings'
import { useLibraryStore } from '../stores/library'
import { useBrowseStore } from '../stores/browse'
import { usePlaylistStore } from '../stores/playlist'
import { ThemeMode } from '../types'

const settingsStore = useSettingsStore()
const libraryStore = useLibraryStore()
const browseStore = useBrowseStore()
const playlistStore = usePlaylistStore()

const isRefreshing = ref(false)
const refreshError = ref('')

function getThemeLabel(): string {
    return settingsStore.theme === ThemeMode.Dark ? '暗色模式' : '亮色模式'
}

async function refreshLibrary() {
    isRefreshing.value = true
    refreshError.value = ''
    try {
        await libraryStore.refreshLibrary()
        browseStore.clearSelection()
        if (playlistStore.currentPlaylistId) {
            await playlistStore.selectPlaylist(playlistStore.currentPlaylistId)
        }
    } catch (e) {
        refreshError.value = '刷新失败, 请检查文件夹是否存在'
        console.error('Refresh failed:', e)
    } finally {
        isRefreshing.value = false
    }
}
</script>

<template>
    <div class="settings-view">
        <h2 class="settings-title">设置</h2>

        <div class="settings-section">
            <h3 class="section-title">外观</h3>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">主题</span>
                    <span class="setting-desc">切换亮色/暗色模式</span>
                </div>
                <button class="setting-action" @click="settingsStore.toggleTheme()">
                    {{ getThemeLabel() }}
                </button>
            </div>
        </div>

        <div class="settings-section">
            <h3 class="section-title">显示</h3>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">乐库视图模式</span>
                    <span class="setting-desc">切换列表视图或卡片视图</span>
                </div>
                <button class="setting-action" @click="settingsStore.toggleViewMode()">
                    {{ settingsStore.viewMode === 'list' ? '列表模式' : '卡片模式' }}
                </button>
            </div>
        </div>

        <div class="settings-section">
            <h3 class="section-title">播放</h3>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">媒体键</span>
                    <span class="setting-desc">支持键盘媒体键控制播放/暂停/上下曲</span>
                </div>
                <button
                    class="setting-toggle"
                    :class="{ active: settingsStore.mediaKeysEnabled }"
                    @click="settingsStore.toggleMediaKeys()"
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>
        </div>

        <div class="settings-section">
            <h3 class="section-title">系统</h3>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">播放中不息屏</span>
                    <span class="setting-desc">播放时阻止系统休眠和显示器关闭</span>
                </div>
                <button
                    class="setting-toggle"
                    :class="{ active: settingsStore.preventSleep }"
                    @click="settingsStore.togglePreventSleep()"
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">最小化到托盘</span>
                    <span class="setting-desc">点击关闭按钮时最小化到系统托盘</span>
                </div>
                <button
                    class="setting-toggle"
                    :class="{ active: settingsStore.closeToTray }"
                    @click="settingsStore.toggleCloseToTray()"
                >
                    <span class="toggle-knob"></span>
                </button>
            </div>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">刷新乐库</span>
                    <span class="setting-desc" :class="{ 'error-text': refreshError }">{{ refreshError || '清除缓存并重新扫描所有音频文件元数据' }}</span>
                </div>
                <button class="setting-action" :disabled="isRefreshing" @click="refreshLibrary()">
                    {{ isRefreshing ? '刷新中...' : '刷新' }}
                </button>
            </div>
        </div>

        <div class="settings-section">
            <h3 class="section-title">关于</h3>
            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">Abel Music Player</span>
                    <span class="setting-desc">版本 0.1.0</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.settings-view {
    padding: 24px 32px;
    overflow-y: auto;
    height: 100%;
}

.settings-title {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 24px;
}

.settings-section {
    margin-bottom: 24px;
}

.section-title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    padding: 0 4px;
}

.setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-radius: var(--radius-md);
    transition: background-color var(--transition-fast);
}

.setting-item:hover {
    background-color: var(--color-bg-secondary);
}

.setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.setting-label {
    font-size: 14px;
    font-weight: 500;
}

.setting-desc {
    font-size: 12px;
    color: var(--color-text-secondary);
}

.error-text {
    color: #e53935;
}

.setting-action {
    padding: 6px 14px;
    border-radius: var(--radius-md);
    background-color: var(--color-bg-hover);
    font-size: 13px;
    transition: background-color var(--transition-fast);
}

.setting-action:hover {
    background-color: var(--color-border);
}

.setting-badge {
    font-size: 12px;
    color: var(--color-accent);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    background-color: var(--color-bg-secondary);
}

.setting-toggle {
    position: relative;
    width: 40px;
    height: 22px;
    border-radius: 11px;
    background-color: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: background-color var(--transition-fast);
    flex-shrink: 0;
}

.setting-toggle.active {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
}

.toggle-knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    transition: transform var(--transition-fast);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.setting-toggle.active .toggle-knob {
    transform: translateX(18px);
}
</style>