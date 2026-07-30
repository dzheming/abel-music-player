<script setup lang="ts">
import { usePlayerStore } from '../stores/player.ts'
import PlayerLayout from '../components/Layout/PlayerLayout.vue'
import CoverDisplay from '../components/Display/CoverDisplay.vue'
import ProgressBar from '../components/Controls/ProgressBar.vue'
import PlayControls from '../components/Controls/PlayControls.vue'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'
import SpectrumVisualizer from '../components/Display/SpectrumVisualizer.vue'

const emit = defineEmits<{ close: [] }>()
const playerStore = usePlayerStore()
</script>

<template>
    <PlayerLayout hide-footer @close="emit('close')">
        <template #default="{ coverGradient }">
            <div class="split-body">
                <div class="split-left">
                    <div class="vinyl-wrapper" :class="{ spinning: playerStore.isPlaying }">
                        <div class="vinyl-disc">
                            <div class="vinyl-cover">
                                <CoverDisplay :gradient="coverGradient" />
                            </div>
                            <div class="vinyl-hole"></div>
                        </div>
                    </div>
                    <div class="left-controls">
                        <SpectrumVisualizer class="spectrum-display"/>
                        <ProgressBar hide-time />
                        <PlayControls />
                    </div>
                </div>
                <div class="split-right">
                    <LyricsDisplay />
                </div>
            </div>
        </template>
    </PlayerLayout>
</template>

<style scoped>
.split-body {
    flex: 1;
    display: grid;
    grid-template-columns: 2fr 3fr;
    min-height: 0;
    padding: 0 48px 24px;
    gap: 48px;
}

.split-left {
    display: grid;
    grid-template-rows: 7fr 3fr;
    min-height: 0;
}

.vinyl-wrapper {
    align-self: end;
    justify-self: center;
    width: min(100%, calc(100vh - 320px));
    max-width: 480px;
    aspect-ratio: 1 / 1;
}

.left-controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 24px;
    width: 100%;
    padding-top: clamp(16px, 1vw, 32px);
}

.left-controls :deep(.play-controls) {
    width: 90%;
    justify-content: space-between;
}

.left-controls :deep(.progress-bar) {
    max-width: 90%;
}

.left-controls :deep(.progress-slider) {
    background: rgba(255, 255, 255, 0.2);
}

.left-controls :deep(.control-btn) {
    color: rgba(255, 255, 255, 0.8);
}

.left-controls :deep(.control-btn:hover) {
    background-color: rgba(255, 255, 255, 0.1);
}

.left-controls :deep(.control-btn.active) {
    color: var(--color-accent);
}

.left-controls :deep(.play-btn) {
    background-color: var(--color-accent);
    color: #fff;
}

.vinyl-disc {
    position: relative;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    background: repeating-radial-gradient(
        circle at center,
        #0d0d0d 0,
        #0d0d0d 1.5px,
        #1a1a1a 2px,
        #0d0d0d 2.5px
    );
}

.vinyl-disc::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: 50%;
    background: conic-gradient(
        from 0deg,
        transparent 0%,
        rgba(255, 100, 100, 0.06) 10%,
        transparent 20%,
        rgba(100, 255, 100, 0.05) 35%,
        transparent 45%,
        rgba(100, 100, 255, 0.06) 60%,
        transparent 70%,
        rgba(255, 255, 100, 0.05) 85%,
        transparent 100%
    );
    mix-blend-mode: overlay;
}

.vinyl-cover {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 55%;
    height: 55%;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    overflow: hidden;
}

.vinyl-cover :deep(.cover-display) {
    border-radius: 50%;
    box-shadow: none;
    -webkit-box-reflect: unset !important;
}

.vinyl-hole {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 8%;
    height: 8%;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    background: #333;
    border: 2px solid #444;
    box-shadow: inset 0 0 4px rgba(0, 0, 0, 0.5);
}

.vinyl-wrapper.spinning .vinyl-disc {
    animation: spin 20s linear infinite;
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

.spectrum-display {
    height: 35%;
    -webkit-box-reflect: below 0px linear-gradient(transparent 20%, rgba(255, 255, 255, 0.5));
}

.split-right {
    overflow: hidden;
    min-height: 0;
}

.split-right :deep(.lyrics-display) {
    padding: 0 24px;
}
</style>
