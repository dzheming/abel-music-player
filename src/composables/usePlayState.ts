import { ref } from 'vue'
import type { Ref } from 'vue'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { toTrack } from '../types'
import type { Track, RawTrack } from '../types'

interface SavedPlayState {
    paths: string[]
    currentIndex: number
    currentTime: number
}

export function usePlayState(
    audio: HTMLAudioElement,
    playlist: Ref<Track[]>,
    currentIndex: Ref<number>,
    currentTime: Ref<number>,
    onTrackRestored: (track: Track, coverUrl: string) => void
) {
    const isRestoringState = ref(true)
    let restoreMetaListener: (() => void) | null = null

    function clearRestoreListener() {
        if (restoreMetaListener) {
            audio.removeEventListener('loadedmetadata', restoreMetaListener)
            restoreMetaListener = null
        }
    }

    function savePlayState() {
        const state: SavedPlayState = {
            paths: playlist.value.map(f => f.path),
            currentIndex: currentIndex.value,
            currentTime: audio.currentTime || 0,
        }
        invoke('set_setting', { key: 'play-state', value: JSON.stringify(state) }).catch(() => {})
    }

    async function restorePlayState() {
        try {
            const raw = await invoke('get_setting', { key: 'play-state' })
            if (!raw) return
            const state: SavedPlayState = JSON.parse(raw as string)
            if (state.paths.length > 0 && state.currentIndex >= 0 && state.currentIndex < state.paths.length) {
                const cached: RawTrack[] = await invoke('get_cached_tracks_for_paths', { paths: state.paths })
                const cachedMap = new Map(cached.map(c => [c.path, c]))
                const restored: Track[] = state.paths.map(path => {
                    const c = cachedMap.get(path)
                    if (c) return toTrack(c)
                    return { path, fileName: path.split(/[/\\]/).pop() || path }
                })
                playlist.value = restored
                currentIndex.value = state.currentIndex
                const track = playlist.value[state.currentIndex]
                if (track) {
                    audio.src = convertFileSrc(track.path)
                    const restoreTime = state.currentTime
                    clearRestoreListener()
                    const onLoadedMeta = () => {
                        audio.currentTime = restoreTime
                        currentTime.value = restoreTime
                        clearRestoreListener()
                    }
                    restoreMetaListener = onLoadedMeta
                    audio.addEventListener('loadedmetadata', onLoadedMeta)
                    invoke<string | null>('read_cover', { path: track.path }).then(cover => {
                        if (cover && playlist.value[state.currentIndex]) {
                            playlist.value[state.currentIndex].coverUrl = cover
                            onTrackRestored(playlist.value[state.currentIndex], cover)
                        }
                    }).catch(() => {})
                }
            }
        } catch (e) {
            console.error('Failed to restore play state:', e)
        }
    }

    return { isRestoringState, savePlayState, restorePlayState, clearRestoreListener }
}
