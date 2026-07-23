<script setup lang="ts">
import { computed, watch, nextTick, ref } from 'vue'
import { usePlaylistStore } from '../stores/playlist'
import { usePlayerStore } from '../stores/player'
import { invoke } from '@tauri-apps/api/core'
import { formatTime, stripExtension } from '../utils/format'
import ContextMenu from '../components/ContextMenu.vue'
import type { MenuItem } from '../components/ContextMenu.vue'

const playlistStore = usePlaylistStore()
const playerStore = usePlayerStore()

const showMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)
const menuTrackPath = ref('')

const selectedPlaylist = computed(() => 
    playlistStore.playlists.find(p => p.id === playlistStore.currentPlaylistId) || null
)

async function playPlaylist(index: number) {
    const tracks = playlistStore.currentTracks
    if (tracks.length === 0) return
    const audioFiles = tracks.map(t => ({
        path: t.path,
        fileName: t.file_name,
        title: t.title || undefined,
        artist: t.artist || undefined,
        album: t.album || undefined,
        duration: t.duration,
    }))
    playerStore.setPlaylist(audioFiles, index)
    playlistStore.playingPlaylistId = playlistStore.currentPlaylistId
    invoke('set_setting', { key: 'playing-playlist-id', value: JSON.stringify(playlistStore.currentPlaylistId) }).catch(() => {})
}

function onTrackContextMenu(e: MouseEvent, path: string) {
    e.preventDefault()
    menuX.value = e.clientX
    menuY.value = e.clientY
    menuTrackPath.value = path
    showMenu.value = true
}

const trackListRef = ref<HTMLElement | null>(null)

watch(() => playerStore.currentTrack?.path, () => {
    nextTick(() => {
        if (!trackListRef.value) return
        const playing = trackListRef.value.querySelector('.track-item.playing') as HTMLElement | null
        if (playing) {
            playing.scrollIntoView({ block: 'center', behavior: 'smooth' })
        }
    })
})

const menuItems = computed<MenuItem[]>(() => {
    const path = menuTrackPath.value
    return [
        {
            label: '从列表移除',
            action: () => {
                if (playlistStore.currentPlaylistId) {
                    playlistStore.removeFromPlaylist(playlistStore.currentPlaylistId, [path])
                }
            },
            danger: true,
        },
    ]
})
</script>

<template>
    <div class="playlist-content">
        <div v-if="!playlistStore.currentPlaylistId" class="empty-state">
            <div class="empty-icon">&#9835;</div>
            <p class="empty-text">在左侧选择一个播放列表</p>
        </div>

        <template v-else>
            <div class="content-header">
                <span class="content-title">
                    {{ selectedPlaylist?.name }}
                    <span class="file-count">({{ playlistStore.currentTracks.length }} 首)</span>
                </span>
            </div>

            <div v-if="playlistStore.isLoading" class="loading-state">
                <p>加载中...</p>
            </div>

            <div v-else-if="playlistStore.currentTracks.length === 0" class="empty-state">
                <div class="empty-icon">📂</div>
                <p class="empty-text">播放列表为空</p>
            </div>

            <div v-else ref="trackListRef" class="track-list">
                <div
                    v-for="(track, index) in playlistStore.currentTracks"
                    :key="track.path"
                    class="track-item"
                    :class="{ playing: playerStore.currentTrack?.path === track.path }"
                    @dblclick="playPlaylist(index)"
                    @contextmenu="onTrackContextMenu($event, track.path)"
                >
                    <span class="track-index">{{ index + 1 }}</span>
                    <span class="track-title">{{ track.title || stripExtension(track.file_name) }}</span>
                    <span class="track-duration">{{ formatTime(track.duration) }}</span>
                </div>
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

<style src="../assets/styles/track-list.css"></style>

<style scoped>
.playlist-content {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.track-item.playing {
    background-color: var(--color-bg-hover);
}

.track-item.playing .track-title {
    color: var(--color-accent);
}
</style>