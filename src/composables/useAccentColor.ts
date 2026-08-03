import { watch } from 'vue'
import type { ComputedRef } from 'vue'
import { extractDominantColorCancelable } from '../utils/extract-color'
import type { Track } from '../types'

interface AccentColorSettings {
    accentColor: string
    applyAccentColor: (color: string) => void
}

export function useAccentColor(
    currentTrack: ComputedRef<Track | null>,
    settingsStore: AccentColorSettings
) {
    let pendingColorExtract: { cancel: () => void } | null = null

    watch(() => currentTrack.value?.coverUrl, (coverUrl) => {
        if (pendingColorExtract) {
            pendingColorExtract.cancel()
            pendingColorExtract = null
        }
        if (coverUrl) {
            const { promise, cancel } = extractDominantColorCancelable(coverUrl)
            pendingColorExtract = { cancel }
            promise.then(color => {
                pendingColorExtract = null
                if (color) settingsStore.applyAccentColor(color)
            })
        } else {
            settingsStore.applyAccentColor(settingsStore.accentColor)
        }
    })
}
