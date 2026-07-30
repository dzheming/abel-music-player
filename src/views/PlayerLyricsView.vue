<script setup lang="ts">
import { computed } from 'vue'
import PlayerLayout from '../components/Player/PlayerLayout.vue'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'
import CoverDisplay from '../components/Player/CoverDisplay.vue'
import { usePlayerStore } from '../stores/player'
import { generateGradient } from '../utils/cover-gradient'
import { stripExtension } from '../utils/format'

const emit = defineEmits<{ close: [] }>()
const playerStore = usePlayerStore()

const displayTitle = computed(() => {
    const track = playerStore.currentTrack
    if (!track) return '未播放'
    return track.title || stripExtension(track.fileName)
})

const displayArtist = computed(() => {
    return playerStore.currentTrack?.artist || ''
})

const hasCover = computed(() => !!playerStore.currentTrack?.coverUrl)

const dynamicBackground = computed(() => {
    if (hasCover.value) return {}
    return { background: generateGradient(displayTitle.value, displayArtist.value) }
})
</script>

<template>
    <PlayerLayout @close="emit('close')">
        <div class="lyrics-view-body" :style="dynamicBackground">
            <div class="lyrics-container">
                <LyricsDisplay />
            </div>
            <div class="floating-cover">
                <CoverDisplay v-if="playerStore.currentTrack?.coverUrl" />
            </div>
        </div>
    </PlayerLayout>
</template>

<style scoped>
.lyrics-view-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 0;
    position: relative;
    padding: 20px 60px;
    transition: background 0.5s ease;
}

.lyrics-container {
    flex: 1;
    width: 100%;
    max-width: 800px;
    min-height: 0;
}

.lyrics-container :deep(.lyrics-display) {
    text-align: center;
    -webkit-mask-image: linear-gradient(
        to bottom,
        transparent 0%,
        black 15%,
        black 85%,
        transparent 100%
    );
    mask-image: linear-gradient(
        to bottom,
        transparent 0%,
        black 15%,
        black 85%,
        transparent 100%
    );
}

.lyrics-container :deep(.lyrics-line) {
    padding: 10px 0;
    font-size: clamp(14px, 1.4vw, 22px);
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.lyrics-container :deep(.lyrics-line.active) {
    font-size: clamp(32px, 2vw, 64px);
    font-weight: 600;
    letter-spacing: 0.02em;
}

.floating-cover {
    position: absolute;
    bottom: 80px;
    right: 60px;
    width: 120px;
    opacity: 0.6;
    transition: opacity 0.3s, transform 0.3s;
}

.floating-cover:hover {
    opacity: 1;
    transform: scale(1.05);
}

.floating-cover :deep(.cover-display) {
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    -webkit-box-reflect: none;
}
</style>
