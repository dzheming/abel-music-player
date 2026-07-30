<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import PlayerLayout from '../components/Player/PlayerLayout.vue'
import CoverDisplay from '../components/Player/CoverDisplay.vue'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'
import SpectrumVisualizer from '../components/Player/SpectrumVisualizer.vue'

const emit = defineEmits<{ close: [] }>()
const coverRef = ref<HTMLElement | null>(null)
const lyricsHeight = ref('300px')

function updateLyricsHeight() {
    if (coverRef.value) {
        lyricsHeight.value = `${coverRef.value.clientHeight * 1.2}px`
    }
}

onMounted(() => {
    nextTick(updateLyricsHeight)
    window.addEventListener('resize', updateLyricsHeight)
})

onUnmounted(() => {
    window.removeEventListener('resize', updateLyricsHeight)
})
</script>

<template>
    <PlayerLayout hide-header-info @close="emit('close')">
        <template #default="{ title, artist, coverGradient }">
            <div class="main-row">
                <div class="title-row">
                    <h1 class="body-title">{{ title }}</h1>
                    <p v-if="artist" class="body-artist">{{ artist }}</p>
                </div>
                <div class="cover-section" ref="coverRef">
                    <CoverDisplay :gradient="coverGradient" />
                </div>
                <div class="lyrics-section" :style="{ height: lyricsHeight }">
                    <LyricsDisplay />
                </div>
            </div>
            <div class="spectrum-row">
                <SpectrumVisualizer />
            </div>
        </template>

        <template #footer-background>
            <div class="spectrum-reflection">
                <SpectrumVisualizer />
            </div>
        </template>
    </PlayerLayout>
</template>

<style scoped>
.main-row {
    flex: 80;
    display: grid;
    grid-template-columns: 31fr 69fr;
    grid-template-rows: auto 1fr;
    align-items: start;
    gap: 16px 1px;
    min-height: 0;
    position: relative;
    z-index: 1;
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
    font-size: clamp(14px, 1.5vw, 20px);
    color: rgba(255, 255, 255, 0.7);
    margin: 4px 0 0;
}

.cover-section {
    display: flex;
    align-items: center;
    justify-content: flex-end;
}

.cover-section :deep(.cover-display) {
    width: min(80%, calc(100vh - 320px), calc(100vw - 400px));
    min-width: 240px;
}

.lyrics-section {
    overflow: hidden;
    min-height: 0;
}

.lyrics-section :deep(.lyrics-display) {
    padding: 0 16px;
    text-align: center;
}

.spectrum-row {
    flex: 20;
    align-items: center;
    justify-content: center;
    min-height: 0;
    z-index: 0;
}

.spectrum-reflection {
    position: absolute;
    inset: 0;
    transform: scaleY(-1);
    opacity: 0.35;
    mask-image: linear-gradient(to top, black 30%, transparent);
    -webkit-mask-image: linear-gradient(to top, black 30%, transparent);
    pointer-events: none;
}
</style>
