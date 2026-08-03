import { stripExtension } from '../utils/format'
import type { Track } from '../types'

interface MediaSessionSettings {
    mediaKeysEnabled: boolean
}

export function useMediaSession(settingsStore: MediaSessionSettings) {
    function update(track: Track) {
        if (!('mediaSession' in navigator)) return
        const artwork = track.coverUrl ? [{ src: track.coverUrl }] : []
        navigator.mediaSession.metadata = new MediaMetadata({
            title: track.title || stripExtension(track.fileName),
            artist: track.artist || '',
            album: track.album || '',
            artwork,
        })
    }

    function setupActionHandlers(togglePlay: () => void, prev: () => void, next: () => void) {
        if (!('mediaSession' in navigator)) return
        navigator.mediaSession.setActionHandler('play', () => { if (settingsStore.mediaKeysEnabled) togglePlay() })
        navigator.mediaSession.setActionHandler('pause', () => { if (settingsStore.mediaKeysEnabled) togglePlay() })
        navigator.mediaSession.setActionHandler('previoustrack', () => { if (settingsStore.mediaKeysEnabled) prev() })
        navigator.mediaSession.setActionHandler('nexttrack', () => { if (settingsStore.mediaKeysEnabled) next() })
    }

    return { update, setupActionHandlers }
}
