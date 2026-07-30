<script setup lang="ts">
import { ref, computed } from 'vue'
import { usePlayerStore } from '../../stores/player'

const playerStore = usePlayerStore()
const previousVolume = ref(playerStore.volume)

function onVolumeChange(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value)
    playerStore.setVolume(value / 100)
}

function toggleMute() {
    if (playerStore.volume > 0) {
        previousVolume.value = playerStore.volume
        playerStore.setVolume(0)
    } else {
        playerStore.setVolume(previousVolume.value || 0.8)
    }
}

const volumeIcon = computed(() => {
    if (playerStore.volume === 0) return 'muted'
    if (playerStore.volume < 0.5) return 'low'
    return 'high'
})
</script>

<template>
    <div class="volume-control">
        <button class="volume-icon" @click="toggleMute" :title="playerStore.volume === 0 ? '取消静音' : '静音'">
            <svg v-if="volumeIcon === 'muted'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                <line x1="23" y1="9" x2="17" y2="15" />
                <line x1="17" y1="9" x2="23" y2="15" />
            </svg>

            <svg v-else-if="volumeIcon === 'low'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
            </svg>

            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
            </svg>
        </button>
        <input
            type="range"
            class="volume-slider"
            min="0"
            max="100"
            step="1"
            :value="playerStore.volume * 100"
            @input="onVolumeChange"
        />
    </div>
</template>

<style scoped>
.volume-control {
    display: flex;
    align-items: center;
    gap: 8px;
}

.volume-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.volume-icon svg {
    width: 20px;
    height: 20px;
}

.volume-icon:hover {
    background-color: var(--color-bg-hover);
}

.volume-slider {
    width: 80px;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--color-border);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
}

.volume-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
}
</style>
