<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { usePlayerStore } from '../../stores/player'
import { stripExtension } from '../../utils/format'
import { generateGradient } from '../../utils/cover-gradient'
import ProgressBar from '../Player/ProgressBar.vue'
import PlayControls from '../Player/PlayControls.vue'
import VolumeControl from '../Player/VolumeControl.vue'
import Equalizer from '../Player/Equalizer.vue'

const emit = defineEmits<{ openPlayer: [], openTimeView: [] }>()
const playerStore = usePlayerStore()
const showEq = ref(false)
const eqBtnRef = ref<HTMLElement | null>(null)
const eqPanelRef = ref<HTMLElement | null>(null)

function toggleEqPanel() {
    showEq.value = !showEq.value
}

function onClickOutside(e: MouseEvent) {
    if (
        showEq.value && 
        eqPanelRef.value && !eqPanelRef.value.contains(e.target as Node) && 
        eqBtnRef.value && !eqBtnRef.value.contains(e.target as Node)
    ) {
        showEq.value = false
    }
}

onMounted(() => document.addEventListener('mousedown', onClickOutside))
onUnmounted(() => document.removeEventListener('mousedown', onClickOutside))

const displayTitle = computed(() => {
    if (playerStore.isRestoringState) return '加载中...'
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
    <footer class="player-bar">
        <ProgressBar class="player-bar-progress" variant="bar" />
        <div class="player-bar-inner">
            <div class="player-bar-left">
                <div class="track-cover" :style="coverGradient" @click="emit('openPlayer')">
                    <img v-if="playerStore.currentTrack?.coverUrl" :src="playerStore.currentTrack.coverUrl" class="cover-img" />
                    <div v-else class="cover-placeholder">&#9835;</div>
                </div>
                <div class="track-info">
                    <span class="track-title" :title="displayTitle">{{ displayTitle }}</span>
                    <span class="track-artist">{{ displayArtist }}</span>
                </div>
            </div>

            <div class="player-bar-center">
                <PlayControls />
            </div>

            <div class="player-bar-right">
                <VolumeControl />
                <div class="eq-wrapper">
                    <button
                        ref="eqBtnRef"
                        class="eq-btn"
                        :class="{ active: playerStore.eqEnabled }"
                        @click="toggleEqPanel"
                        title="均衡器"
                    >EQ</button>
                    <div v-if="showEq" ref="eqPanelRef" class="eq-panel">
                        <Equalizer />
                    </div>
                </div>
                <button class="expand-btn" @click="emit('openTimeView')" title="展开播放页">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="15 3 21 3 21 9" /><polyline points="9 21 3 21 3 15" />
                        <line x1="21" y1="3" x2="14" y2="10" /><line x1="3" y1="21" x2="10" y2="14" />
                    </svg>
                </button>
            </div>
        </div>
    </footer>
</template>

<style scoped>
.player-bar {
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-player-bar);
    border-top: 1px solid var(--color-border);
    transition: background-color var(--transition-normal);
}

.player-bar-progress {
    order: -1;
}

.player-bar-inner {
    display: grid;
    grid-template-columns: 1fr 2fr 1fr;
    align-items: center;
    padding: 0 16px;
    height: var(--player-bar-height);
}

.player-bar-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
}

.track-cover {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    flex-shrink: 0;
    background-color: var(--color-bg-secondary);
    cursor: pointer;
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
    font-size: 20px;
    color: var(--color-text-tertiary);
}

.track-info {
    display: flex;
    flex-direction: column;
    min-width: 0;
}

.track-title {
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.track-artist {
    font-size: 12px;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.player-bar-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.player-bar-right {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
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
    color: var(--color-text-tertiary);
    background: none;
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.eq-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
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
    box-shadow: 0 -4px 16px rgba(0, 0, 0, 0.2);
    z-index: 1000;
}

.expand-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.expand-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
}

.expand-btn svg {
    width: 16px;
    height: 16px;
}
</style>