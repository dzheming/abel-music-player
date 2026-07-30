<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import PlayerLayout from '../components/Player/PlayerLayout.vue'
import LyricsDisplay from '../components/Lyrics/LyricsDisplay.vue'

const emit = defineEmits<{ close: [] }>()

const timeH = ref('')
const timeM = ref('')
let timer: ReturnType<typeof setInterval> | null = null

function updateTime() {
    const now = new Date()
    timeH.value = String(now.getHours()).padStart(2, '0')
    timeM.value = String(now.getMinutes()).padStart(2, '0')
}

onMounted(() => {
    updateTime()
    timer = setInterval(updateTime, 1000)
})

onUnmounted(() => {
    if (timer) clearInterval(timer)
})
</script>

<template>
    <PlayerLayout hide-footer @close="emit('close')">
        <template #header="{ title, artist }">
            <div class="header-info" data-tauri-drag-region>
                <span class="header-title" data-tauri-drag-region>{{ title }}</span>
                <span v-if="artist" class="header-artist" data-tauri-drag-region> - {{ artist }}</span>
            </div>
        </template>

        <div class="time-body">
            <div class="time-section">
                <span class="time-hours">{{ timeH }}</span>
                <span class="time-colon">:</span>
                <span class="time-minutes">{{ timeM }}</span>
            </div>
            <div class="lyrics-section">
                <LyricsDisplay />
            </div>
        </div>
    </PlayerLayout>
</template>

<style scoped>
.header-info {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.header-title {
    font-size: 16px;
    font-weight: 600;
    color: #fff;
}

.header-artist {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.7);
}

.time-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.time-section {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: 'TimeFont';
    font-size: clamp(160px, 12vw, 240px);
    font-weight: 600;
    line-height: 1;
    color: rgba(255, 255, 255, 0.9);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.1em;
}

.time-hours {
    flex: 1;
    text-align: right;
    -webkit-box-reflect: below -0.3em linear-gradient(transparent 40%, rgba(255, 255, 255, 0.2));
}

.time-colon {
    font-weight: 100;
    padding: 0 0.2em;
    -webkit-box-reflect: below -0.3em linear-gradient(transparent 40%, rgba(255, 255, 255, 0.2));
}

.time-minutes {
    flex: 1;
    text-align: left;
    -webkit-box-reflect: below -0.3em linear-gradient(transparent 40%, rgba(255, 255, 255, 0.2));
}

.lyrics-section {
    flex: 1;
    overflow: hidden;
    min-height: 0;
}

.lyrics-section :deep(.lyrics-display) {
    height: 100%;
    padding: 0 48px;
    text-align: center;
}
</style>
