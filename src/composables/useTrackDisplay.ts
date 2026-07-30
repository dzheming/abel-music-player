import { computed } from 'vue'
import { usePlayerStore } from '../stores/player'
import { stripExtension } from '../utils/format'
import { generateGradient } from '../utils/cover-gradient'

export function useTrackDisplay() {
    const playerStore = usePlayerStore()

    const displayTitle = computed(() => {
        if (playerStore.isRestoringState) return '加载中...'
        const track = playerStore.currentTrack
        if (!track) return '未播放'
        return track.title || stripExtension(track.fileName)
    })

    const displayArtist = computed(() => {
        return playerStore.currentTrack?.artist || ''
    })

    const coverGradient = computed(() => {
        if (!playerStore.currentTrack || playerStore.currentTrack.coverUrl) return {}
        return { background: generateGradient(displayTitle.value, displayArtist.value) }
    })

    return { displayTitle, displayArtist, coverGradient }
}
