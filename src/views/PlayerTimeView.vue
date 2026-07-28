<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { usePlayerStore } from '../stores/player'
import { generateGradient } from '../utils/cover-gradient'
import { stripExtension } from '../utils/format'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'
import PlayControls from '../components/Player/PlayControls.vue'
import ProgressBar from '../components/Player/ProgressBar.vue'
import VolumeControl from '../components/Player/VolumeControl.vue'

const emit = defineEmits<{ close: [] }>()
const playerStore = usePlayerStore()

const timeH = ref('')
const timeM = ref('')
const timeS = ref('')
let timer: ReturnType<typeof setInterval> | null = null

function updateTime() {
    const now = new Date()
    timeH.value = String(now.getHours()).padStart(2, '0')
    timeM.value = String(now.getMinutes()).padStart(2, '0')
    timeS.value = String(now.getSeconds()).padStart(2, '0')
}

onMounted(() => {
    updateTime()
    timer = setInterval(updateTime, 1000)
})

onUnmounted(() => {
    if (timer) clearInterval(timer)
})

const displayTitle = computed(() => {
    const track = playerStore.currentTrack
    if (!track) return '未播放'
    return track.title || stripExtension(track.fileName)
})

const displayArtist = computed(() => {
    return playerStore.currentTrack?.artist || ''
})

const coverGradient = computed(() => {
    if (!playerStore.currentTrack || playerStore.currentTrack.coverUrl) return {}
    return { background: generateGradient(displayTitle.value, displayArtist.value) }
})
</script>

<template>
    <div class="player-time-view">
        <div class="bg-blur" v-if="playerStore.currentTrack?.coverUrl">
            <img :src="playerStore.currentTrack.coverUrl" class="bg-img" />
        </div>
        <div class="bg-overlay" :class="{ 'overlay-dark': !playerStore.currentTrack?.coverUrl }" :style="coverGradient"></div>

        <div class="time-content">
            <div class="time-header" data-tauri-drag-region>
                <div class="header-info" data-tauri-drag-region>
                    <span class="header-title" data-tauri-drag-region>{{ displayTitle }}</span>
                    <span v-if="displayArtist" class="header-artist" data-tauri-drag-region> - {{ displayArtist }}</span>
                </div>
                <button class="close-btn" @click="emit('close')" title="关闭">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>
                </button>
            </div>

            <div class="time-body">
                <div class="time-section">
                    <span class="time-hours">{{ timeH }}</span>
                    <span class="time-colon">:</span>
                    <span class="time-minutes">{{ timeM }}</span>
                </div>

                <div class="lyrics-section">
                    <LyricsDisplay />
                </div>
            </div>

            <div class="time-footer">
                <PlayControls />
                <ProgressBar />
                <VolumeControl />
            </div>
        </div>
    </div>
</template>

<style scoped>
.player-time-view {
    position: fixed;
    inset: 0;
    z-index: 1000;
    overflow: hidden;
}

.bg-blur {
    position: absolute;
    inset: -50px;
    z-index: 0;
}

.bg-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    filter: blur(50px) saturate(1.2) brightness(0.65);
    transform: scale(1.2);
}

.bg-overlay {
    position: absolute;
    inset: 0;
    z-index: 1;
    background: var(--player-overlay-color, rgba(0, 0, 0, 0.3));
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
}

.bg-overlay.overlay-dark {
    background: rgba(0, 0, 0, 0.5);
}

.time-content {
    position: relative;
    z-index: 2;
    display: flex;
    flex-direction: column;
    height: 100%;
}

.time-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 24px;
    flex-shrink: 0;
}

.header-info {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.header-title {
    font-size: 16px;
    font-weight: 600;
    color: #fff;
}

.header-artist {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.7);
}

.close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    color: rgba(255, 255, 255, 0.7);
    flex-shrink: 0;
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.close-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: #fff;
}

.close-btn svg {
    width: 18px;
    height: 18px;
}

.time-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.time-section {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'TimeFont';
    font-size: clamp(160px, 12vw, 240px);
    font-weight: 600;
    line-height: 1;
    color: rgba(255, 255, 255, 0.9);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.1em;
}

.time-hours {
    flex: 1;
    text-align: right;
    -webkit-box-reflect: below -0.3em linear-gradient(transparent 40%, rgba(255, 255, 255, 0.2));
}

.time-colon {
    font-weight: 100;
    padding: 0 0.2em;
    -webkit-box-reflect: below -0.3em linear-gradient(transparent 40%, rgba(255, 255, 255, 0.2));
}

.time-minutes {
    flex: 1;
    text-align: left;
    -webkit-box-reflect: below -0.3em linear-gradient(transparent 40%, rgba(255, 255, 255, 0.2));
}

.lyrics-section {
    flex: 1;
    overflow: hidden;
    min-height: 0;
    -webkit-mask-image: linear-gradient(
        to bottom,
        transparent 0%,
        rgba(0, 0, 0, 1) 10%,
        rgba(0, 0, 0, 1) 90%,
        transparent 100%
    );
    mask-image: linear-gradient(
        to bottom,
        transparent 0%,
        rgba(0, 0, 0, 1) 10%,
        rgba(0, 0, 0, 1) 90%,
        transparent 100%
    );
}

.lyrics-section :deep(.lyrics-display) {
    height: 100%;
    padding: 0 48px;
    text-align: center;
}

.time-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 24px;
    padding: 8px 48px 16px;
    flex-shrink: 0;
}

.time-footer :deep(.control-btn) {
    color: rgba(255, 255, 255, 0.8);
}

.time-footer :deep(.control-btn:hover) {
    background-color: rgba(255, 255, 255, 0.1);
}

.time-footer :deep(.control-btn.active) {
    color: var(--color-accent);
}

.time-footer :deep(.play-btn) {
    background-color: var(--color-accent);
    color: #fff;
}

.time-footer :deep(.time-label) {
    color: rgba(255, 255, 255, 0.6);
}

.time-footer :deep(.progress-slider) {
    background: rgba(255, 255, 255, 0.2);
}

.time-footer :deep(.volume-icon) {
    color: rgba(255, 255, 255, 0.8);
}

.time-footer :deep(.volume-slider) {
    background: rgba(255, 255, 255, 0.2);
}
</style>