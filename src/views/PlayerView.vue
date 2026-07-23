<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, nextTick } from 'vue'
import { usePlayerStore } from '../stores/player'
import { stripExtension } from '../utils/format'
import { generateGradient } from '../utils/cover-gradient'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'
import SpectrumVisualizer from '../components/Player/SpectrumVisualizer.vue'
import PlayControls from '../components/Player/PlayControls.vue'
import ProgressBar from '../components/Player/ProgressBar.vue'
import VolumeControl from '../components/Player/VolumeControl.vue'
import Equalizer from '../components/Player/Equalizer.vue'

const emit = defineEmits<{ close: [] }>()
const playerStore = usePlayerStore()
const coverRef = ref<HTMLElement | null>(null)
const lyricsHeight = ref('300px')
const showEq = ref(false)
const eqBtnRef = ref<HTMLElement | null>(null)
const eqPanelRef = ref<HTMLElement | null>(null)

function toggleEqPanel() {
    showEq.value = !showEq.value
}

function onEqClickOutside(e: MouseEvent) {
    if (
        showEq.value && 
        eqPanelRef.value && !eqPanelRef.value.contains(e.target as Node) && 
        eqBtnRef.value && !eqBtnRef.value.contains(e.target as Node)
    ) {
        showEq.value = false
    }
}

function updateLyricsHeight() {
    if (coverRef.value) {
        lyricsHeight.value = `${coverRef.value.clientHeight}px`
    }
}

onMounted(() => {
    nextTick(updateLyricsHeight)
    window.addEventListener('resize', updateLyricsHeight)
    document.addEventListener('mousedown', onEqClickOutside)
})

onUnmounted(() => {
    window.removeEventListener('resize', updateLyricsHeight)
    document.removeEventListener('mousedown', onEqClickOutside)
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
    <div class="player-view">
        <div class="bg-blur" v-if="playerStore.currentTrack?.coverUrl">
            <img :src="playerStore.currentTrack.coverUrl" class="bg-img" />
        </div>
        <div class="bg-overlay" :class="{ 'overlay-dark': !playerStore.currentTrack?.coverUrl }"></div>

        <div class="player-view-content">
            <div class="player-view-header" data-tauri-drag-region>
                <div class="header-info" data-tauri-drag-region>
                    <!--
                    <span class="header-title" data-tauri-drag-region>{{ displayTitle }}</span>
                    <span v-if="displayArtist" class="header-artist" data-tauri-drag-region> - {{ displayArtist }}</span>
                    -->
                </div>
                <button class="close-btn" @click="emit('close')" title="关闭">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18"/>
                    </svg>
                </button>
            </div>

            <div class="player-view-body">
                <div class="main-row">
                    <div class="title-row">
                        <h1 class="body-title">{{ displayTitle }}</h1>
                        <p v-if="displayArtist" class="body-artist">{{ displayArtist }}</p>
                    </div>
                    <div class="cover-section">
                        <div class="cover-large" ref="coverRef" :style="coverGradient">
                            <img v-if="playerStore.currentTrack?.coverUrl" :src="playerStore.currentTrack.coverUrl" class="cover-img" />
                            <div v-else class="cover-placeholder">&#9834;</div>
                        </div>
                    </div>
                    <div class="lyrics-section" :style="{ height: lyricsHeight }">
                        <LyricsDisplay />
                    </div>
                </div>
                <div class="spectrum-row">
                    <SpectrumVisualizer />
                </div>
            </div>

            <div class="player-view-footer">
                <PlayControls />
                <ProgressBar />
                <VolumeControl />
                <div class="eq-wrapper">
                    <button 
                        ref="eqBtnRef"
                        class="eq-btn"
                        :class="{ active: playerStore.eqEnabled }"
                        @click="toggleEqPanel"
                    >EQ</button>
                    <div v-if="showEq" ref="eqPanelRef" class="eq-panel">
                        <Equalizer />
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.player-view {
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

.player-view-content {
    position: relative;
    z-index: 2;
    display: flex;
    flex-direction: column;
    height: 100%;
}

.player-view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
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

.player-view-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 0 48px;
    min-height: 0;
    position: relative;
}

.title-row {
    grid-column: 1 / -1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
}

.body-title {
    font-size: clamp(40px, 3.5vw, 80px);
    font-weight: 700;
    color: #fff;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
}

.body-artist {
    font-size: clamp(15px, 1.6vw, 20px);
    color: rgba(255, 255, 255, 0.5);
    margin: 4px 0 0 0;
}

.main-row {
    flex: 80;
    display: grid;
    grid-template-columns: 31fr 69fr;
    grid-template-rows: auto 1fr;
    align-items: center;
    gap: 16px 32px;
    min-height: 0;
    position: relative;
    z-index: 1;
}

.spectrum-row {
    flex: 20;
    align-items: center;
    justify-content: center;
    min-height: 0;
    z-index: 0;
}

.cover-section {
    display: flex;
    align-items: center;
    justify-content: center;
}

.cover-large {
    width: min(80%, calc(100vh - 320px));
    min-width: 240px;
    aspect-ratio: 1;
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
}

.cover-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.cover-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 64px;
    color: rgba(255, 255, 255, 0.3);
    background-color: rgba(255, 255, 255, 0.05);
}

.lyrics-section {
    overflow: hidden;
    min-height: 0;
    -webkit-mask-image: linear-gradient(
        to bottom,
        transparent 0%,
        rgba(0, 0, 0, 1) 25%,
        rgba(0, 0, 0, 1) 75%,
        transparent 100%
    );
    mask-image: linear-gradient(
        to bottom,
        transparent 0%,
        rgba(0, 0, 0, 1) 25%,
        rgba(0, 0, 0, 1) 75%,
        transparent 100%
    );
}

.lyrics-section :deep(.lyrics-display) {
    height: 100%;
    padding: 0 16px;
    text-align: center;
}

.player-view-footer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 24px;
    padding: 8px 48px 16px;
    flex-shrink: 0;
}

.player-view-footer :deep(.control-btn) {
    color: rgba(255, 255, 255, 0.8);
}

.player-view-footer :deep(.control-btn:hover) {
    background-color: rgba(255, 255, 255, 0.1);
}

.player-view-footer :deep(.control-btn.active) {
    color: var(--color-accent);
}

.player-view-footer :deep(.play-btn) {
    background-color: var(--color-accent);
    color: #fff;
}

.player-view-footer :deep(.time-label) {
    color: rgba(255, 255, 255, 0.6);
}

.player-view-footer :deep(.progress-slider) {
    background: rgba(255, 255, 255, 0.2);
}

.player-view-footer :deep(.volume-icon) {
    color: rgba(255, 255, 255, 0.8);
}

.player-view-footer :deep(.volume-slider) {
    background: rgba(255, 255, 255, 0.2);
}

.eq-wrapper {
    position: relative;
}

.eq-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    font-size: 15px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.5);
    background: none;
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.eq-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.85);
}

.eq-btn.active {
    color: var(--color-accent);
}

.eq-panel {
    position: absolute;
    bottom: 100%;
    right: 0;
    margin-bottom: 8px;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 -4px 16px rgba(0, 0, 0, 0.3);
    z-index: 1000;
}
</style>