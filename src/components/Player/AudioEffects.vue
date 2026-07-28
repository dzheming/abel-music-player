<script setup lang="ts">
import { usePlayerStore } from '../../stores/player'

const playerStore = usePlayerStore()

function onSpeedChange(e: Event) {
    playerStore.setPlaybackSpeed(Number((e.target as HTMLInputElement).value))
}

function resetSpeed() {
    playerStore.setPlaybackSpeed(1.0)
}

function onBalanceChange(e: Event) {
    playerStore.setStereoBalance(Number((e.target as HTMLInputElement).value))
}

function onReverbChange(e: Event) {
    playerStore.setReverbMix(Number((e.target as HTMLInputElement).value))
}

function onBassChange(e: Event) {
    playerStore.setBassBoost(Number((e.target as HTMLInputElement).value))
}

function onVocalChange(e: Event) {
    playerStore.setVocalBoost(Number((e.target as HTMLInputElement).value))
}

function formatBalance(v: number): string {
    if (v === 0) return 'C'
    return v < 0 ? `L${Math.abs(v)}` : `R${v}`
}
</script>

<template>
    <div class="audio-effects">
        <div class="effect-row">
            <span class="effect-label">速度</span>
            <input
                type="range"
                class="effect-slider"
                min="0.5"
                max="2.0"
                step="0.05"
                :value="playerStore.playbackSpeed"
                @input="onSpeedChange"
                @dblclick="resetSpeed"
            />
            <span class="effect-value">{{ playerStore.playbackSpeed.toFixed(2) }}x</span>
        </div>

        <div class="effect-row">
            <span class="effect-label">平衡</span>
            <input
                type="range"
                class="effect-slider"
                min="-100"
                max="100"
                step="1"
                :value="playerStore.stereoBalance"
                @input="onBalanceChange"
            />
            <span class="effect-value">{{ formatBalance(playerStore.stereoBalance) }}</span>
        </div>

        <div class="effect-row">
            <span class="effect-label">混响</span>
            <input
                type="range"
                class="effect-slider"
                min="0"
                max="100"
                step="1"
                :value="playerStore.reverbMix"
                @input="onReverbChange"
            />
            <span class="effect-value">{{ playerStore.reverbMix }}%</span>
        </div>

        <div class="effect-row">
            <span class="effect-label">低音</span>
            <input
                type="range"
                class="effect-slider"
                min="0"
                max="100"
                step="1"
                :value="playerStore.bassBoost"
                @input="onBassChange"
            />
            <span class="effect-value">{{ playerStore.bassBoost }}</span>
        </div>

        <div class="effect-row">
            <span class="effect-label">人声</span>
            <input
                type="range"
                class="effect-slider"
                min="0"
                max="100"
                step="1"
                :value="playerStore.vocalBoost"
                @input="onVocalChange"
            />
            <span class="effect-value">{{ playerStore.vocalBoost }}</span>
        </div>

        <button class="reset-btn" @click="playerStore.resetEffects()">重置</button>
    </div>
</template>

<style scoped>
.audio-effects {
    padding: 12px;
    width: 320px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.effect-row {
    display: flex;
    align-items: center;
    gap: 8px;
}

.effect-label {
    font-size: 12px;
    color: var(--color-text-secondary);
    width: 32px;
    flex-shrink: 0;
}

.effect-slider {
    flex: 1;
    height: 4px;
    cursor: pointer;
    accent-color: var(--color-accent);
}

.effect-value {
    font-size: 11px;
    color: var(--color-text-tertiary);
    width: 40px;
    text-align: right;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
}

.reset-btn {
    align-self: flex-end;
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    background-color: var(--color-bg-hover);
    color: var(--color-text-secondary);
    transition: all var(--transition-fast);
}

.reset-btn:hover {
    background-color: var(--color-accent);
    color: #fff;
}
</style>