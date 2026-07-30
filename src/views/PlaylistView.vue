<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlaylistStore } from '../stores/playlist'
import { usePlayerStore } from '../stores/player'
import { useLibraryStore } from '../stores/library'
import { toTrack } from '../types'
import type { RawTrack } from '../types'
import ContextMenu from '../components/ContextMenu.vue'
import type { MenuItem } from '../components/ContextMenu.vue'

const playlistStore = usePlaylistStore()
const playerStore = usePlayerStore()
const libraryStore = useLibraryStore()

watch(() => playlistStore.playlists, (list) => {
    if (!playlistStore.currentPlaylistId && list.length > 0) {
        const playingId = playlistStore.playingPlaylistId
        if (playingId && list.some(p => p.id === playingId)) {
            playlistStore.selectPlaylist(playingId)
        } else {
            playlistStore.selectPlaylist(list[0].id)
        }
    }
}, { immediate: true })

const showCreateDialog = ref(false)
const newPlaylistName = ref('')
const editingId = ref<number | null>(null)
const editingName = ref('')
const showMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)
const menuPlaylistId = ref<number | null>(null)

function startCreate() {
    showCreateDialog.value = true
    newPlaylistName.value = ''
}

async function confirmCreate() {
    if (!newPlaylistName.value.trim()) return
    await playlistStore.createPlaylist(newPlaylistName.value.trim())
    showCreateDialog.value = false
}

function startRename(id: number, name: string) {
    editingId.value = id
    editingName.value = name
}

async function confirmRename() {
    if (editingId.value !== null && editingName.value.trim()) {
        await playlistStore.renamePlaylist(editingId.value, editingName.value.trim())
    }
    editingId.value = null
}

async function handleDelete(id: number) {
    await playlistStore.deletePlaylist(id)
}

function onPlaylistContextMenu(e: MouseEvent, id: number) {
    e.preventDefault()
    menuX.value = e.clientX
    menuY.value = e.clientY
    menuPlaylistId.value = id
    showMenu.value = true
}

function selectPlaylist(id: number) {
    playlistStore.selectPlaylist(id)
}

async function addCurrentFolderToPlaylist(playlistId: number) {
    const paths = libraryStore.audioFiles.map(f => f.path)
    if (paths.length > 0) {
        await playlistStore.addToPlaylist(playlistId, paths)
    }
}

const playlistMenuItems = computed<MenuItem[]>(() => {
    const id = menuPlaylistId.value
    if (!id) return []
    const pl = playlistStore.playlists.find(p => p.id === id)
    if (!pl) return []
    const isDefault = id === playlistStore.defaultPlaylistId

    const items: MenuItem[] = []

    if (!isDefault) {
        items.push({ label: '重命名', action: () => startRename(id, pl.name) })
    }

    items.push({ label: '添加当前目录', action: () => addCurrentFolderToPlaylist(id) })
    items.push({ label: '清空列表', action: () => playlistStore.clearPlayList(id), danger: true })

    if (!isDefault) {
        items.push({ label: '删除', action: () => handleDelete(id), danger: true })
    }

    return items
})

const isRandomPlaying = ref(false)

async function randomPlay() {
    isRandomPlaying.value = true
    try {
        await playlistStore.ensureDefaultPlaylist()
        const defaultId = playlistStore.defaultPlaylistId
        if (!defaultId) return

        await playlistStore.clearPlayList(defaultId)

        const tracks: RawTrack[] = await invoke('get_random_tracks', { count: 100 })
        if (tracks.length === 0) return

        const paths = tracks.map(t => t.path)
        await playlistStore.addToPlaylist(defaultId, paths)

        playerStore.shuffle = true
        playerStore.setPlaylist(tracks.map(toTrack), 0)

        if (playlistStore.currentPlaylistId !== defaultId) {
            await playlistStore.selectPlaylist(defaultId)
        }
    } catch (e) {
        console.error('Random play failed:', e)

    } finally {
        isRandomPlaying.value = false
    }
}
</script>

<template>
    <div class="playlist-view">
        <div class="view-header">
            <button class="create-btn random-btn" :class="{ loading: isRandomPlaying }" :disabled="isRandomPlaying" @click="randomPlay">
                {{ isRandomPlaying ? '加载中...' : '随机播放' }}
            </button>
            <button class="create-btn" @click="startCreate">新建列表</button>
        </div>

        <div v-if="showCreateDialog" class="create-dialog">
            <input
                v-model="newPlaylistName"
                type="text"
                class="create-input"
                placeholder="名称..."
                @keyup.enter="confirmCreate"
                @keyup.escape="showCreateDialog = false"
                autofocus
            />
            <button class="confirm-btn" @click="confirmCreate">确定</button>
        </div>

        <div class="playlist-list">
            <div
                v-for="pl in playlistStore.playlists"
                :key="pl.id"
                class="playlist-item"
                :class="{ active: playlistStore.currentPlaylistId === pl.id, playing: playlistStore.playingPlaylistId === pl.id }"
                @click="selectPlaylist(pl.id)"
                @contextmenu="onPlaylistContextMenu($event, pl.id)"
            >
                <template v-if="editingId === pl.id">
                    <input
                        v-model="editingName"
                        type="text"
                        class="rename-input"
                        @keyup.enter="confirmRename"
                        @keyup.escape="editingId = null"
                        @blur="confirmRename"
                        autofocus
                    />
                </template>
                <template v-else>
                    <span class="playing-icon" v-if="playlistStore.playingPlaylistId === pl.id">&#9654;</span>
                    <span class="playlist-name">{{ pl.name }}</span>
                    <span class="playlist-count">{{ pl.track_count }}</span>
                </template>
            </div>
        </div>

        <div v-if="playlistStore.playlists.length === 0" class="empty-hint">
            暂无播放列表
        </div>

        <ContextMenu
            v-if="showMenu"
            :x="menuX"
            :y="menuY"
            :items="playlistMenuItems"
            @close="showMenu = false"
        />
    </div>
</template>

<style scoped>
.playlist-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.view-header {
    padding: 8px 8px 4px;
    flex-shrink: 0;
}

.create-btn {
    width: 100%;
    padding: 5px 12px;
    border-radius: var(--radius-md);
    font-size: 14px;
    color: var(--color-text-secondary);
    text-align: left;
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.create-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
}

.random-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.random-btn.loading {
    color: var(--color-accent);
    opacity: 1;
}

.create-dialog {
    display: flex;
    gap: 4px;
    padding: 4px 8px 8px;
    flex-shrink: 0;
}

.create-input, .rename-input {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 12px;
    outline: none;
    min-width: 0;
}

.create-input:focus, .rename-input:focus {
    border-color: var(--color-accent);
}

.confirm-btn {
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    background-color: var(--color-accent);
    color: #fff;
    font-size: 12px;
    flex-shrink: 0;
}

.playlist-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 4px;
}

.playlist-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color var(--transition-fast);
    font-size: 13px;
}

.playlist-item:hover {
    background-color: var(--color-bg-hover);
}

.playlist-item.active {
    background-color: var(--color-bg-hover);
    font-weight: 500;
}

.playlist-item.playing .playlist-name {
    color: var(--color-accent);
}

.playing-icon {
    font-size: 9px;
    color: var(--color-accent);
    flex-shrink: 0;
}

.playlist-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
}

.playlist-count {
    font-size: 11px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
    margin-left: 8px;
}

.empty-hint {
    padding: 16px;
    text-align: center;
    font-size: 12px;
    color: var(--color-text-tertiary);
}
</style>
