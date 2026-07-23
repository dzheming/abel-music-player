<script setup lang="ts">
import { ref, computed } from 'vue'
import { useLibraryStore } from '../stores/library'
import { usePlaylistStore } from '../stores/playlist'
import { usePlayerStore } from '../stores/player'
import { useSettingsStore } from '../stores/settings'
import { useSidebarTab } from '../composables/useSidebarTab'
import { formatTime, stripExtension } from '../utils/format'
import ContextMenu from '../components/ContextMenu.vue'
import MusicCard from '../components/Library/MusicCard.vue'
import PlaylistContent from './PlaylistContent.vue'
import BrowseContent from './BrowseContent.vue'
import { useTrackContextMenu } from '../composables/useTrackContextMenu'
import type { AudioFile } from '../types'

const libraryStore = useLibraryStore()
const playlistStore = usePlaylistStore()
const playerStore = usePlayerStore()
const settingsStore = useSettingsStore()
const { activeTab } = useSidebarTab()
const localSearch = ref('')
const menuTrackPath = ref('')
const { showMenu, menuX, menuY, onContextMenu: triggerMenu, menuItems } = useTrackContextMenu(() => menuTrackPath.value)

const displayFiles = computed(() => {
    if (libraryStore.globalSearchQuery) {
        return libraryStore.globalSearchResults
    }
    const files = libraryStore.audioFiles
    if (!localSearch.value.trim()) return files
    const q = localSearch.value.toLowerCase()
    return files.filter(f => {
        const title = (f.title || stripExtension(f.fileName)).toLowerCase()
        return title.includes(q)
    })
})

const collapsedDirs = ref<Set<string>>(new Set())

function toggleDir(dir: string) {
    if (collapsedDirs.value.has(dir)) {
        collapsedDirs.value.delete(dir)
    } else {
        collapsedDirs.value.add(dir)
    }
}

interface GroupedFiles {
    dir: string
    files: { file: typeof displayFiles.value[number]; globalIndex: number }[]
}

const groupedFiles = computed<GroupedFiles[]>(() => {
    const groups: GroupedFiles[] = []
    let currentDir = ''
    let currentGroup: GroupedFiles | null = null

    displayFiles.value.forEach((file, index) => {
        const dir = file.path.replace(/[/\\][^/\\]+$/, '')
        if (dir !== currentDir) {
            currentDir = dir
            currentGroup = { dir, files: [] }
            groups.push(currentGroup)
        }
        currentGroup!.files.push({ file, globalIndex: index })
    })

    for (const group of groups) {
        group.files.sort((a, b) => (a.file.trackNumber ?? Infinity) - (b.file.trackNumber ?? Infinity))
    }

    return groups
})

const showingGlobalSearch = computed(() => !!libraryStore.globalSearchQuery)

const folderName = computed(() => {
    const p = libraryStore.selectedFolderPath
    if (!p) return '乐库'
    return p.replace(/[/\\]$/, '').split(/[/\\]/).pop() || '乐库'
})

function onTrackDblClick(file: AudioFile) {
    playlistStore.addToDefault([file.path])
    playerStore.appendTracks([file])
    const trackIndex = playerStore.playlist.findIndex(f => f.path === file.path)
    if (trackIndex >= 0) {
        playerStore.playTrackAt(trackIndex)
    }
}

function onTrackContextMenu(e: MouseEvent, path: string) {
    menuTrackPath.value = path
    triggerMenu(e)
}

function isPlaying(path: string) {
    return playerStore.currentTrack?.path === path && playerStore.isPlaying
}
</script>

<template>
    <div class="home-view">
        <PlaylistContent v-if="activeTab === 'playlist'" />
        <BrowseContent v-else-if="activeTab === 'browse'" />

        <div v-else-if="libraryStore.folders.length === 0" class="empty-state">
            <div class="empty-icon">🎵</div>
            <p class="empty-text">请添加音乐文件夹开始使用</p>
            <button class="empty-btn" @click="libraryStore.addFolder()">+ 添加文件夹</button>
        </div>

        <template v-else>
            <div class="content-header">
                <span class="content-title">
                    {{ showingGlobalSearch ? '搜索结果' : folderName }}
                    <span v-if="displayFiles.length > 0" class="file-count">({{ displayFiles.length }})</span>
                </span>
                <div class="header-actions">
                    <div class="local-search-box" v-if="!showingGlobalSearch && libraryStore.audioFiles.length > 0">
                        <span class="search-icon">&#128269;</span>
                        <input
                            v-model="localSearch"
                            type="text"
                            class="search-input"
                            placeholder="搜索当前列表..."
                        />
                        <button v-if="localSearch" class="search-clear" @click="localSearch = ''">&times;</button>
                    </div>
                    <button class="view-toggle-btn" @click="settingsStore.toggleViewMode()" :title="settingsStore.viewMode === 'list' ? '切换卡片视图' : '切换列表视图'">
                        <span v-if="settingsStore.viewMode === 'list'">&#9783;</span>
                        <span v-else>&#9776;</span>
                    </button>
                </div>
            </div>

            <div v-if="libraryStore.isScanning || libraryStore.isGlobalSearching" class="loading-state">
                <p>{{ libraryStore.scanProgress || '扫描中...' }}</p>
            </div>

            <div v-else-if="displayFiles.length === 0" class="empty-state">
                <div class="empty-icon">🎵</div>
                <p class="empty-text">{{ showingGlobalSearch ? '未找到匹配的音乐' : '没有找到音频文件' }}</p>
            </div>

            <div v-else-if="settingsStore.viewMode === 'list'" class="music-list">
                <div v-for="group in groupedFiles" :key="group.dir" class="dir-group">
                    <div class="dir-separator" @click="toggleDir(group.dir)">
                        <span class="dir-arrow" :class="{ collapsed: collapsedDirs.has(group.dir) }">&#9662;</span>
                        <span class="dir-path">{{ group.dir }}</span>
                        <span class="dir-count">{{ group.files.length }}</span>
                        <span class="dir-line"></span>
                    </div>
                    <template v-if="!collapsedDirs.has(group.dir)">
                        <div
                            v-for="item in group.files"
                            :key="item.file.path"
                            class="track-item"
                            :class="{ playing: isPlaying(item.file.path) }"
                            @dblclick="onTrackDblClick(item.file)"
                            @contextmenu="onTrackContextMenu($event, item.file.path)"
                        >
                            <span class="track-index">{{ item.file.trackNumber || '-' }}</span>
                            <span class="track-title">{{ item.file.title || stripExtension(item.file.fileName) }}</span>
                            <span class="track-album">{{ item.file.album || '' }}</span>
                            <span class="track-duration">{{ formatTime(item.file.duration || 0) }}</span>
                        </div>
                    </template>
                </div>
            </div>

            <div v-else class="music-grid-view">
                <template v-for="group in groupedFiles" :key="group.dir">
                    <div class="dir-separator" @click="toggleDir(group.dir)">
                        <span class="dir-arrow" :class="{ collapsed: collapsedDirs.has(group.dir) }">&#9662;</span>
                        <span class="dir-path">{{ group.dir }}</span>
                        <span class="dir-count">{{ group.files.length }}</span>
                        <span class="dir-line"></span>
                    </div>
                    <div v-if="!collapsedDirs.has(group.dir)" class="music-grid">
                        <MusicCard
                            v-for="item in group.files"
                            :key="item.file.path"
                            :file="item.file"
                            @dblclick="onTrackDblClick(item.file)"
                        />
                    </div>
                </template>
            </div>

            <ContextMenu
                v-if="showMenu"
                :x="menuX"
                :y="menuY"
                :items="menuItems"
                @close="showMenu = false"
            />
        </template>
    </div>
</template>

<style scoped>
.home-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.content-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px 8px;
    flex-shrink: 0;
}

.content-title {
    font-size: 16px;
    font-weight: 600;
}

.file-count {
    font-size: 13px;
    font-weight: 400;
    color: var(--color-text-secondary);
}

.local-search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border-radius: var(--radius-md);
    background-color: var(--color-bg-secondary);
    transition: background-color var(--transition-fast);
}

.local-search-box:focus-within {
    outline: 1px solid var(--color-accent);
}

.search-icon {
    font-size: 12px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
}

.search-input {
    border: none;
    background: none;
    outline: none;
    font-size: 12px;
    color: var(--color-text-primary);
    font-family: inherit;
    width: 140px;
}

.search-input::placeholder {
    color: var(--color-text-tertiary);
}

.search-clear {
    font-size: 14px;
    color: var(--color-text-tertiary);
    padding: 0 2px;
    flex-shrink: 0;
}

.search-clear:hover {
    color: var(--color-text-primary);
}

.header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.view-toggle-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    font-size: 16px;
    color: var(--color-text-secondary);
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.view-toggle-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    gap: 12px;
}

.empty-icon {
    font-size: 48px;
}

.empty-text {
    font-size: 14px;
    color: var(--color-text-secondary);
}

.empty-btn {
    margin-top: 8px;
    padding: 8px 16px;
    border-radius: var(--radius-md);
    background-color: var(--color-accent);
    color: #fff;
    font-size: 13px;
    font-weight: 500;
    transition: background-color var(--transition-fast);
}

.empty-btn:hover {
    background-color: var(--color-accent-hover);
}

.loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--color-text-secondary);
}

.music-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 16px 16px;
}

.dir-group {
    margin-bottom: 4px;
}

.dir-separator {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    cursor: pointer;
    user-select: none;
    transition: background-color var(--transition-fast);
}

.dir-separator:hover {
    background-color: var(--color-bg-hover);
    border-radius: var(--radius-sm);
}

.dir-arrow {
    font-size: 10px;
    color: var(--color-text-tertiary);
    transition: transform var(--transition-fast);
    flex-shrink: 0;
}

.dir-arrow.collapsed {
    transform: rotate(-90deg);
}

.dir-path {
    font-size: 13px;
    color: var(--color-text-secondary);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.dir-count {
    font-size: 11px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
}

.dir-line {
    flex: 1;
    height: 1px;
    background-color: var(--color-border);
}

.track-item {
    display: grid;
    grid-template-columns: 32px 1fr auto 60px;
    align-items: center;
    gap: 12px;
    padding: 7px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color var(--transition-fast);
}

.track-item:hover {
    background-color: var(--color-bg-hover);
}

.track-item.playing {
    background-color: var(--color-bg-hover);
}

.track-item.playing .track-title {
    color: var(--color-accent);
}

.track-index {
    font-size: 12px;
    color: var(--color-text-tertiary);
    text-align: center;
}

.track-title {
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
}

.track-album {
    font-size: 12px;
    color: var(--color-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.track-duration {
    font-size: 12px;
    color: var(--color-text-tertiary);
    text-align: right;
}

.music-grid-view {
    flex: 1;
    overflow-y: auto;
    padding: 0 16px 16px;
}

.music-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 4px;
    align-content: start;
}
</style>