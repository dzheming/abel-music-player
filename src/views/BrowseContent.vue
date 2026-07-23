<script setup lang="ts">
import { ref, computed } from 'vue'
import { useBrowseStore } from '../stores/browse'
import { usePlayerStore } from '../stores/player'
import { usePlaylistStore } from '../stores/playlist'
import { formatTime, stripExtension } from '../utils/format'
import ContextMenu from '../components/ContextMenu.vue'
import { useTrackContextMenu } from '../composables/useTrackContextMenu'

const browseStore = useBrowseStore()
const playerStore = usePlayerStore()
const playlistStore = usePlaylistStore()

const menuTrackPath = ref('')
const { showMenu, menuX, menuY, onContextMenu: triggerMenu, menuItems } = useTrackContextMenu(() => menuTrackPath.value)

const title = computed(() => {
    if (browseStore.currentArtist) return browseStore.currentArtist
    if (browseStore.currentAlbum) return browseStore.currentAlbum
    return ''
})

function playTrack(index: number) {
    const track = browseStore.tracks[index]
    if (!track) return
    playlistStore.addToDefault([track.path])
    playerStore.appendTracks([track])
    const trackIndex = playerStore.playlist.findIndex(f => f.path === track.path)
    if (trackIndex >= 0) {
        playerStore.playTrackAt(trackIndex)
    }
}

function onTrackContextMenu(e: MouseEvent, path: string) {
    menuTrackPath.value = path
    triggerMenu(e)
}
</script>

<template>
    <div class="browse-content">
        <div v-if="!browseStore.currentArtist && !browseStore.currentAlbum" class="empty-state">
            <div class="empty-icon">&#9776;</div>
            <p class="empty-text">在左侧选择歌手或专辑</p>
        </div>

        <template v-else>
            <div class="content-header">
                <span class="content-title">
                    {{ title }}
                    <span class="file-count">({{ browseStore.tracks.length }} 首)</span>
                </span>
            </div>

            <div v-if="browseStore.isLoading" class="loading-state">
                <p>加载中...</p>
            </div>

            <div v-else class="track-list">
                <div
                    v-for="(track, index) in browseStore.tracks"
                    :key="track.path"
                    class="track-item"
                    @dblclick="playTrack(index)"
                    @contextmenu="onTrackContextMenu($event, track.path)"
                >
                    <span class="track-index">{{ index + 1 }}</span>
                    <span class="track-title">{{ track.title || stripExtension(track.fileName) }}</span>
                    <span class="track-duration">{{ formatTime(track.duration || 0) }}</span>
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
.browse-content {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
</style>