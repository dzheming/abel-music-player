<script setup lang="ts">
import { computed, ref } from 'vue'
import { usePlayerStore } from '../../stores/player'
import { formatTime } from '../../utils/format'

const props = defineProps<{
    variant?: 'slider' | 'bar'
    hideTime?: boolean
}>()

const variant = computed(() => props.variant ?? 'slider')

const playerStore = usePlayerStore()

const isDragging = ref(false)
const dragValue = ref(0)

const progressPercent = computed(() => {
    if (isDragging.value) return dragValue.value * 100
    return playerStore.progress * 100
})

function onSeekStart(e: Event) {
    isDragging.value = true
    dragValue.value = parseFloat((e.target as HTMLInputElement).value) / 100
}

function onSeekMove(e: Event) {
    if (!isDragging.value) return
    dragValue.value = parseFloat((e.target as HTMLInputElement).value) / 100
}

function onSeekEnd(e: Event) {
    if (!isDragging.value) return
    const value = parseFloat((e.target as HTMLInputElement).value)
    isDragging.value = false
    playerStore.seek(value / 100)
}

function onBarClick(e: MouseEvent) {
    const bar = e.currentTarget as HTMLElement
    const rect = bar.getBoundingClientRect()
    const fraction = (e.clientX - rect.left) / rect.width
    playerStore.seek(fraction)
}
</script>

<template>
    <div v-if="variant === 'slider'" class="progress-bar">
        <span v-if="!props.hideTime" class="time-label">{{ formatTime(playerStore.currentTime) }}</span>
        <input
            type="range"
            class="progress-slider"
            min="0"
            max="100"
            step="0.1"
            :value="progressPercent"
            @input="onSeekMove"
            @mousedown="onSeekStart"
            @touchstart="onSeekStart"
            @change="onSeekEnd"
        />
        <span v-if="!props.hideTime" class="time-label">{{ formatTime(playerStore.duration) }}</span>
    </div>

    <div v-else class="progress-bar-bar" @click="onBarClick">
        <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
    </div>
</template>

<style scoped>
.progress-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 500px;
}

.time-label {
    font-size: 11px;
    color: var(--color-text-secondary);
    min-width: 32px;
    text-align: center;
}

.progress-slider {
    flex: 1;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--color-border);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
}

.progress-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    margin-top: -4px;
    transition: transform var(--transition-fast);
}

.progress-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
}

.progress-slider::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
}

.progress-slider::-moz-range-track {
    height: 4px;
    background: var(--color-border);
    border-radius: 2px;
    border: none;
}

.progress-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-accent);
    border: none;
    cursor: pointer;
}

.progress-slider::-moz-range-thumb:hover {
    transform: scale(1.2);
}

.progress-bar-bar {
    width: 100%;
    height: 4px;
    background: var(--color-border);
    cursor: pointer;
    position: relative;
    flex-shrink: 0;
}

.progress-fill {
    height: 100%;
    background: var(--color-accent);
    border-radius: 0 2px 2px 0;
    transition: width 0.1s linear;
}
</style>
