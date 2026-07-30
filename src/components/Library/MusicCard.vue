<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { usePlayerStore } from '../../stores/player'
import { formatTime } from '../../utils/format'
import { stripExtension } from '../../utils/format'
import { generateGradient } from '../../utils/cover-gradient'
import ContextMenu from '../ContextMenu.vue'
import { useTrackContextMenu } from '../../composables/useTrackContextMenu'
import type { Track } from '../../types'

const props = defineProps<{
    file: Track
    mode?: 'library' | 'list'
}>()

const emit = defineEmits<{ dblclick: [] }>()

const playerStore = usePlayerStore()

const { showMenu, menuX, menuY, onContextMenu, menuItems } = useTrackContextMenu(() => props.file.path)
const lazyCover = ref<string | undefined>(props.file.coverUrl)
const cardRef = ref<HTMLElement | null>(null)

const displayTitle = computed(() => {
    return props.file.title || stripExtension(props.file.fileName)
})

const isCurrentTrack = computed(() => {
    return playerStore.currentTrack?.path === props.file.path
})

const gradientStyle = computed(() => {
    if (lazyCover.value) return {}
    return { background: generateGradient(displayTitle.value, props.file.artist) }
})

onMounted(() => {
    if (lazyCover.value) return
    const observer = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting) {
            observer.disconnect()
            invoke<string | null>('read_cover', { path: props.file.path }).then(cover => {
                if (cover) lazyCover.value = cover
            }).catch(() => {})
        }
    }, { rootMargin: '200px' })
    if (cardRef.value) observer.observe(cardRef.value)
})

function onDblClick() {
    emit('dblclick')
}
</script>

<template>
    <div ref="cardRef" class="music-card" :class="{ playing: isCurrentTrack }" @contextmenu="onContextMenu" @dblclick="onDblClick">
        <div class="card-cover" :style="gradientStyle">
            <img v-if="lazyCover" :src="lazyCover" class="cover-img" />
            <div v-else class="cover-placeholder">&#9835;</div>
            <div v-if="isCurrentTrack && playerStore.isPlaying" class="playing-indicator">
                <span></span><span></span><span></span>
            </div>
            <span v-if="file.duration" class="card-duration">{{ formatTime(file.duration) }}</span>
        </div>
        <div class="card-info">
            <span class="card-title" :title="displayTitle">{{ displayTitle }}</span>
            <span v-if="file.artist" class="card-artist">{{ file.artist }}</span>
            <span v-if="file.album" class="card-album">{{ file.album }}</span>
        </div>
        <ContextMenu
            v-if="showMenu"
            :x="menuX"
            :y="menuY"
            :items="menuItems"
            @close="showMenu = false"
        />
    </div>
</template>

<style scoped>
.music-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: background-color var(--transition-fast), transform var(--transition-fast);
}

.music-card:hover {
    background-color: var(--color-bg-hover);
}

.music-card.playing {
    outline: 1.5px solid var(--color-accent);
    outline-offset: -1.5px;
}

.card-cover {
    position: relative;
    width: 100%;
    aspect-ratio: 1;
    border-radius: var(--radius-md);
    overflow: hidden;
    background-color: var(--color-bg-secondary);
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
    font-size: 32px;
    color: var(--color-text-tertiary);
}

.playing-indicator {
    position: absolute;
    bottom: 8px;
    right: 8px;
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 16px;
}

.playing-indicator span {
    display: block;
    width: 3px;
    background-color: var(--color-accent);
    border-radius: 1px;
    animation: equalize 0.6s infinite alternate;
}

.playing-indicator span:nth-child(1) { height: 8px; animation-delay: 0s; }
.playing-indicator span:nth-child(2) { height: 12px; animation-delay: 0.2s; }
.playing-indicator span:nth-child(3) { height: 6px; animation-delay: 0.4s; }

@keyframes equalize {
    from { height: 4px; }
    to { height: 16px; }
}

.card-duration {
    position: absolute;
    bottom: 6px;
    left: 6px;
    font-size: 10px;
    padding: 2px 5px;
    border-radius: var(--radius-sm);
    background-color: rgba(0, 0, 0, 0.6);
    color: #fff;
}

.card-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.card-title {
    font-size: 12px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.card-artist {
    font-size: 11px;
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.card-album {
    font-size: 10px;
    color: var(--color-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
