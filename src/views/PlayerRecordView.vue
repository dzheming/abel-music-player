<script setup lang="ts">
import { usePlayerStore } from '../stores/player.ts'
import PlayerLayout from '../components/Player/PlayerLayout.vue'
import CoverDisplay from '../components/Player/CoverDisplay.vue'
import ProgressBar from '../components/Player/ProgressBar.vue'
import PlayControls from '../components/Player/PlayControls.vue'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'

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
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: clamp(48px, 1.5vw, 120px);
    min-height: 0;
}

.left-controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 36px;
    width: 100%;
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

.vinyl-wrapper {
    width: min(80%, calc(100vh - 320px), calc(100vw - 400px));
    max-width: 480px;
    aspect-ratio: 1;
}

.vinyl-disc {
    position: relative;
    width: 100%;
    height: 100%;
    border-radius: 50%;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    background: radial-gradient(
        circle,
        transparent 0%,
        transparent 29%,
        #1a1a1a 29.5%,
        #111 35%,
        #1a1a1a 40%,
        #0d0d0d 45%,
        #1a1a1a 50%,
        #111 55%,
        #1a1a1a 60%,
        #0d0d0d 65%,
        #1a1a1a 70%,
        #111 80%,
        #1a1a1a 90%,
        #222 100%
    );
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

.split-right {
    overflow: hidden;
    min-height: 0;
}

.split-right :deep(.lyrics-display) {
    padding: 0 24px;
}
</style>
