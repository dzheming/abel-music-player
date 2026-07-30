<script setup lang="ts">
import { ref, watch, computed, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlayerStore } from '../../stores/player'
import { parseLrc, findCurrentLine } from '../../utils/lrc-parser'
import type { LrcLine } from '../../utils/lrc-parser'
import LyricsSearchDialog from './LyricsSearchDialog.vue'
import ContextMenu from '../ContextMenu.vue'

const playerStore = usePlayerStore()
const lyricsLines = ref<LrcLine[]>([])
const currentLineIndex = ref(-1)
const containerRef = ref<HTMLElement | null>(null)
const spacerHeight = ref('50vh')
const isLoading = ref(false)
const noLyrics = ref(false)
const showSearchDialog = ref(false)
const showContextMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)

function onContextMenu(e: MouseEvent) {
    if (!playerStore.currentTrack) return
    menuX.value = e.clientX
    menuY.value = e.clientY
    showContextMenu.value = true
}

function onLyricsSelected(lrc: string) {
    lyricsLines.value = parseLrc(lrc)
    noLyrics.value = lyricsLines.value.length === 0
    showSearchDialog.value = false
}

const currentLine = computed(() =>
    findCurrentLine(lyricsLines.value, playerStore.currentTime)
)

watch(currentLine, (idx) => {
    if (idx !== currentLineIndex.value) {
        currentLineIndex.value = idx
        scrollToLine(idx)
    }
})

watch(() => playerStore.currentTrack, async (track) => {
    lyricsLines.value = []
    currentLineIndex.value = -1
    noLyrics.value = false

    if (!track) return
    await loadLyrics(track.path, track.title, track.artist, track.album, track.duration)
}, { immediate: true })

onMounted(() => {
    updateSpacerHeight()
    window.addEventListener('resize', onResize)
})

onUnmounted(() => {
    window.removeEventListener('resize', onResize)
})

function onResize() {
    updateSpacerHeight()
    nextTick(() => {
        scrollToLine(currentLineIndex.value)
    })
}

function updateSpacerHeight() {
    if (containerRef.value) {
        spacerHeight.value = `${containerRef.value.clientHeight / 2}px`
    }
}

async function loadLyrics(
    path: string,
    title?: string,
    artist?: string,
    album?: string,
    duration?: number
) {
    isLoading.value = true
    try {
        const local: string | null = await invoke('read_local_lyrics', { audioPath: path })
        if (local) {
            lyricsLines.value = parseLrc(local)
            if (lyricsLines.value.length > 0) return
        }

        if (title && artist) {
            try {
                const downloaded = await Promise.race([
                    invoke<string | null>('download_lyrics', {
                        title: title || '',
                        artist: artist || '',
                        album: album || '',
                        duration: duration || 0,
                        audioPath: path,
                    }),
                    new Promise<null>((resolve) => setTimeout(() => resolve(null), 5000))
                ]) as string | null
                if (downloaded) {
                    lyricsLines.value = parseLrc(downloaded)
                    if (lyricsLines.value.length > 0) return
                }
            } catch (e) {

            }
        }
        noLyrics.value = true
    } catch (e) {
        console.error('Failed to load lyrics:', e)
        noLyrics.value = true
    } finally {
        isLoading.value = false
    }
}

function scrollToLine(index: number) {
    if (index < 0 || !containerRef.value) return
    nextTick(() => {
        const lineEl = containerRef.value?.querySelector(`[data-line="${index}"]`) as HTMLElement
        if (!lineEl || !containerRef.value) return
        const containerHeight = containerRef.value.clientHeight
        const scrollTop = lineEl.offsetTop - containerHeight / 2 + lineEl.clientHeight / 2
        containerRef.value.scrollTo({ top: scrollTop, behavior: 'smooth' })
    })
}
</script>

<template>
    <div class="lyrics-display" ref="containerRef" @contextmenu.prevent="onContextMenu">
        <div v-if="isLoading" class="lyrics-status">歌词加载中...</div>
        <div v-else-if="noLyrics" class="lyrics-status lyrics-clickable" @click="showSearchDialog = true">暂无歌词,点击搜索</div>
        <div v-else-if="lyricsLines.length === 0 && !playerStore.currentTrack" class="lyrics-status">未播放</div>
        <div v-else class="lyrics-content">
            <div class="lyrics-spacer" :style="{ height: spacerHeight }"></div>
            <p
                v-for="(line, index) in lyricsLines"
                :key="index"
                :data-line="index"
                class="lyrics-line"
                :class="{ active: index === currentLineIndex, past: index < currentLineIndex }"
            >{{ line.text || '...' }}</p>
            <div class="lyrics-spacer" :style="{ height: spacerHeight }"></div>
        </div>

        <ContextMenu
            v-if="showContextMenu"
            :x="menuX"
            :y="menuY"
            :items="[{ label: '搜索歌词', action: () => { showContextMenu = false; showSearchDialog = true } }]"
            @close="showContextMenu = false"
        />

        <LyricsSearchDialog
            v-if="showSearchDialog"
            :title="playerStore.currentTrack?.title || ''"
            :album="playerStore.currentTrack?.album || ''"
            :artist="playerStore.currentTrack?.artist || ''"
            :audio-path="playerStore.currentTrack?.path || ''"
            @select="onLyricsSelected"
            @close="showSearchDialog = false"
        />
    </div>
</template>

<style scoped>
.lyrics-display {
    height: 100%;
    overflow-y: auto;
    scrollbar-width: none;
    -webkit-mask-image: linear-gradient(to bottom, transparent 0%, black 10%, black 90%, transition 100%);
    mask-image: linear-gradient(to bottom, transparent 0%, black 10%, black 90%, transition 100%);
}

.lyrics-display::-webkit-scrollbar {
    display: none;
}

.lyrics-status {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-tertiary);
    font-size: 14px;
}

.lyrics-clickable {
    cursor: pointer;
    transition: color 0.15s;
}

.lyrics-clickable:hover {
    color: var(--color-accent);
}

.lyrics-content {
    position: relative;
}

.lyrics-line {
    padding: 8px 0;
    font-size: clamp(12px, 1.5vw, 26px);
    line-height: 1.2;
    color: rgba(255, 255, 255, 0.3);
    transition: color 0.3s ease, font-size 0.3s ease;
    cursor: default;
}

.lyrics-line.past {
    color: rgba(255, 255, 255, 0.1);
}


.lyrics-line.active {
    font-size: clamp(32px, 1.8vw, 64px);
    font-weight: 500;
    color: color-mix(in srgb, var(--color-accent) 80%, transparent);
}
</style>
