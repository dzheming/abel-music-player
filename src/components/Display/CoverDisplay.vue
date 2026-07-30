<script setup lang="ts">
import { usePlayerStore } from '../../stores/player'

import type { CSSProperties } from 'vue'
defineProps<{ gradient?: CSSProperties }>()
const playerStore = usePlayerStore()
</script>

<template>
    <div class="cover-display" :style="gradient">
        <img v-if="playerStore.currentTrack?.coverUrl" :src="playerStore.currentTrack.coverUrl" class="cover-img" />
        <div v-else class="cover-placeholder">&#9834;</div>
    </div>
</template>

<style scoped>
.cover-display {
    width: 100%;
    aspect-ratio: 1;
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
    -webkit-box-reflect: below 4px linear-gradient(transparent 80%, rgba(255, 255, 255, 0.25));
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
    font-size: 80px;
    color: rgba(255, 255, 255, 0.3);
}
</style>
