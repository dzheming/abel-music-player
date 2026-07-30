<script setup lang="ts">
import { usePlayerStore } from '../../stores/player'
import { LoopMode } from '../../types'

const playerStore = usePlayerStore()

function getLoopTitle(): string {
    switch (playerStore.loopMode) {
        case LoopMode.RepeatOne: return '单曲循环'
        case LoopMode.RepeatAll: return '全部循环'
        default: return '不循环'
    }
}
</script>

<template>
    <div class="play-controls">
        <button
            class="control-btn"
            :class="{ active: playerStore.shuffle }"
            :title="playerStore.shuffle ? '随机播放: 开' : '随机播放: 关'"
            @click="playerStore.toggleShuffle()"
        >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="16 3 21 3 21 8" /><line x1="4" y1="20" x2="21" y2="3" />
                <polyline points="21 16 21 21 16 21" /><line x1="15" y1="15" x2="21" y2="21" />
                <line x1="4" y1="4" x2="9" y2="9" />
            </svg>
        </button>

        <button class="control-btn" title="上一曲" @click="playerStore.prev()">
            <svg viewBox="0 0 24 24" fill="currentColor">
                <rect x="3" y="5" width="2.5" height="14" rx="1" />
                <polygon points="21 5 9 12 21 19" />
            </svg>
        </button>

        <button class="control-btn play-btn" @click="playerStore.togglePlay()">
            <svg v-if="playerStore.isPlaying" viewBox="0 0 24 24" fill="currentColor">
                <rect x="5" y="4" width="5" height="16" rx="1.5" />
                <rect x="14" y="4" width="5" height="16" rx="1.5" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="currentColor">
                <polygon points="6 3 21 12 6 21" />
            </svg>
        </button>

        <button class="control-btn" title="下一曲" @click="playerStore.next()">
            <svg viewBox="0 0 24 24" fill="currentColor">
                <polygon points="3 5 15 12 3 19" />
                <rect x="18.5" y="5" width="2.5" height="14" rx="1" />
            </svg>
        </button>

        <button
            class="control-btn"
            :class="{ active: playerStore.loopMode !== LoopMode.None }"
            :title="getLoopTitle()"
            @click="playerStore.cycleLoopMode()"
        >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="17 1 21 5 17 9" />
                <path d="M3 11V9a4 4 0 0 1 4-4h14" />
                <polyline points="7 23 3 19 7 15" />
                <path d="M21 13v2a4 4 0 0 1-4 4H3" />
            </svg>
            <span v-if="playerStore.loopMode === LoopMode.RepeatOne" class="loop-badge">1</span>
        </button>
    </div>
</template>

<style scoped>
.play-controls {
    display: flex;
    align-items: center;
    gap: 8px;
}

.control-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    transition: background-color var(--transition-fast), transform var(--transition-fast);
    color: var(--color-text-primary);
}

.control-btn svg {
    width: 16px;
    height: 16px;
}

.control-btn:hover {
    background-color: var(--color-bg-hover);
}

.control-btn:active {
    transform: scale(0.92);
}

.play-btn {
    width: 40px;
    height: 40px;
    background-color: var(--color-accent);
    color: #fff;
    border-radius: 50%;
}

.play-btn:hover {
    background-color: var(--color-accent-hover);
}

.play-btn svg {
    width: 18px;
    height: 18px;
}

.control-btn.active {
    color: var(--color-accent);
}

.loop-badge {
    position: absolute;
    bottom: 2px;
    right: 2px;
    font-size: 8px;
    font-weight: 700;
    color: var(--color-accent);
    line-height: 1;
}
</style>