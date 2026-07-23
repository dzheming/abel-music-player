<script setup lang="ts">
import { usePlayerStore } from '../../stores/player'
import { EQ_PRESETS } from '../../composables/useEqualizer'

const playerStore = usePlayerStore()

function applyPreset(preset: { name: string; gains: number[] }) {
    playerStore.setEqPreset(preset.name, preset.gains)
}

function onSliderChange(i: number, value: number) {
    playerStore.setEqGain(i, value)
}

function formatFreq(freq: number): string {
    return freq >= 1000 ? `${freq / 1000}k` : String(freq)
}
</script>

<template>
    <div class="equalizer">
        <div class="eq-header">
            <button
                class="eq-toggle"
                :class="{ active: playerStore.eqEnabled }"
                @click="playerStore.toggleEq()"
            >{{  playerStore.eqEnabled ? 'ON' : 'OFF' }}</button>
            <select
                class="eq-preset-select"
                :value="playerStore.eqPreset"
                @change="applyPreset(EQ_PRESETS.find(p => p.name === ($event.target as HTMLSelectElement).value) || EQ_PRESETS[0])"
            >
                <option v-for="p in EQ_PRESETS" :key="p.name" :value="p.name">{{ p.name }}</option>
                <option v-if="playerStore.eqPreset === '自定义'" value="自定义">自定义</option>
            </select>
        </div>
        <div class="eq-bands">
            <div v-for="(freq, i) in playerStore.EQ_FREQUENCIES" :key="freq" class="eq-band">
                <span class="eq-value">{{ playerStore.eqGains[i] > 0 ? '+' : '' }}{{ playerStore.eqGains[i] }}</span>
                <input
                    type="range"
                    class="eq-slider"
                    min="-12"
                    max="12"
                    step="1"
                    :value="playerStore.eqGains[i]"
                    :disabled="!playerStore.eqEnabled"
                    orient="vertical"
                    @input="onSliderChange(i, Number(($event.target as HTMLInputElement).value))"
                />
                <span class="eq-freq">{{ formatFreq(freq) }}</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.equalizer {
    padding: 12px;
    width: 320px;
}

.eq-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
}

.eq-toggle {
    padding: 3px 10px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
    background-color: var(--color-bg-hover);
    color: var(--color-text-tertiary);
    transition: all var(--transition-fast);
}

.eq-toggle.active {
    background-color: var(--color-accent);
    color: #fff;
}

.eq-preset-select {
    flex: 1;
    padding: 4px 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 12px;
    outline: none;
    cursor: pointer;
}

.eq-preset-select:focus {
    border-color: var(--color-accent);
}

.eq-bands {
    display: flex;
    justify-content: space-between;
    gap: 2px;
}

.eq-band {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.eq-slider {
    writing-mode: vertical-lr;
    direction: rtl;
    width: 18px;
    height: 80px;
    appearance: slider-vertical;
    cursor: pointer;
    accent-color: var(--color-accent);
}

.eq-slider:disabled {
    opacity: 0.3;
    cursor: not-allowed;
}

.eq-value {
    font-size: 9px;
    color: var(--color-text-tertiary);
    min-width: 20px;
    text-align: center;
}

.eq-freq {
    font-size: 9px;
    color: var(--color-text-tertiary);
}
</style>