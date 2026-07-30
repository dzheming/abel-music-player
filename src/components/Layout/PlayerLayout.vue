<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { usePlayerStore } from '../../stores/player'
import { stripExtension } from '../../utils/format'
import { generateGradient } from '../../utils/cover-gradient'
import PlayControls from '../Controls/PlayControls.vue'
import ProgressBar from '../Controls/ProgressBar.vue'
import VolumeControl from '../Controls/VolumeControl.vue'
import EffectsPanel from '../Effects/EffectsPanel.vue'

const props = defineProps<{ hideFooter?: boolean; hideHeaderInfo?: boolean }>()
const emit = defineEmits<{ close: [] }>()
const playerStore = usePlayerStore()

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

onMounted(() => document.addEventListener('mousedown', onEqClickOutside))
onUnmounted(() => document.removeEventListener('mousedown', onEqClickOutside))

const VIEW_STYLES = ['default', 'time', 'lyrics', 'record']

function cycleViewStyle() {
    const idx = VIEW_STYLES.indexOf(playerStore.playerViewStyle)
    playerStore.playerViewStyle = VIEW_STYLES[(idx + 1) % VIEW_STYLES.length]
}

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
    <div class="player-layout">
        <div class="bg-blur" v-if="playerStore.currentTrack?.coverUrl">
            <img :src="playerStore.currentTrack.coverUrl" class="bg-img" />
        </div>
        <div class="bg-overlay" :class="{ 'overlay-dark': !playerStore.currentTrack?.coverUrl }" :style="coverGradient"></div>

        <div class="layout-content">
            <div class="layout-header" data-tauri-drag-region>
                <slot name="header" :title="displayTitle" :artist="displayArtist">
                    <div class="header-info" data-tauri-drag-region>
                        <template v-if="!props.hideHeaderInfo">
                            <span class="header-title" data-tauri-drag-region>{{ displayTitle }}</span>
                            <span v-if="displayArtist" class="header-artist" data-tauri-drag-region> - {{ displayArtist }}</span>
                        </template>
                    </div>
                </slot>
                <div class="header-actions">
                    <button class="style-btn" @click="cycleViewStyle" title="切换风格">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            <rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>
                        </svg>
                    </button>
                    <button class="close-btn" @click="emit('close')" title="关闭">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18"/>
                        </svg>
                    </button>
                </div>
            </div>

            <div class="layout-body">
                <slot :title="displayTitle" :artist="displayArtist" :cover-gradient="coverGradient"></slot>
            </div>

            <div v-if="props.hideFooter" class="layout-footer-spacer"></div>
            <div v-else class="layout-footer">
                <slot name="footer-background"></slot>
                <div class="footer-controls">
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
                            <EffectsPanel />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.player-layout {
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

.layout-content {
    position: relative;
    z-index: 2;
    display: flex;
    flex-direction: column;
    height: 100%;
}

.layout-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 24px;
    flex-shrink: 0;
}

.header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
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

.style-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    color: rgba(255, 255, 255, 0.7);
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.style-btn:hover {
    background-color: rgba(255, 255, 255, 0.1);
    color: #fff;
}

.style-btn svg {
    width: 16px;
    height: 16px;
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

.layout-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    position: relative;
}

.layout-footer-spacer {
    height: 44px;
    flex-shrink: 0;
}

.layout-footer {
    position: relative;
    flex-shrink: 0;
    padding: 8px 48px 16px;
}

.footer-controls {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 24px;
}

.layout-footer :deep(.control-btn) {
    color: rgba(255, 255, 255, 0.8);
}

.layout-footer :deep(.control-btn:hover) {
    background-color: rgba(255, 255, 255, 0.1);
}

.layout-footer :deep(.control-btn.active) {
    color: var(--color-accent);
}

.layout-footer :deep(.play-btn) {
    background-color: var(--color-accent);
    color: #fff;
}

.layout-footer :deep(.time-label) {
    color: rgba(255, 255, 255, 0.6);
}

.layout-footer :deep(.progress-slider) {
    background: rgba(255, 255, 255, 0.2);
}

.layout-footer :deep(.volume-icon) {
    color: rgba(255, 255, 255, 0.8);
}

.layout-footer :deep(.volume-slider) {
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
