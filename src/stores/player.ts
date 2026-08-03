import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { LoopMode } from '../types'
import { useEqualizer, EQ_FREQUENCIES } from '../composables/useEqualizer'
import { useAudioEffects } from '../composables/useAudioEffects'
import { useAudioGraph } from '../composables/useAudioGraph'
import { useAccentColor } from '../composables/useAccentColor'
import { useTaskbarIcon } from '../composables/useTaskbarIcon'
import { usePlayState } from '../composables/usePlayState'
import { useMediaSession } from '../composables/useMediaSession'
import { useSettingsPersistence } from '../composables/useSettingsPersistence'
import { pickNextIndex, pickPrevIndex } from '../composables/usePlaylistNavigation'
import { useSettingsStore } from './settings'
import type { Track } from '../types'

export const usePlayerStore = defineStore('player', () => {
    const audio = new Audio()
    audio.crossOrigin = 'anonymous'
    const playlist = ref<Track[]>([])
    const currentIndex = ref(-1)
    const isPlaying = ref(false)
    const currentTime = ref(0)
    const duration = ref(0)
    const volume = ref(0.8)
    const shuffle = ref(false)
    const loopMode = ref<LoopMode>(LoopMode.None)
    const playerViewStyle = ref('default')

    useSettingsPersistence({ volume, shuffle, loopMode, playerViewStyle })

    const eq = useEqualizer()
    const effects = useAudioEffects()
    const audioGraph = useAudioGraph()
    const taskbarIcon = useTaskbarIcon()
    const settingsStore = useSettingsStore()
    const mediaSession = useMediaSession(settingsStore)

    const currentTrack = computed(() =>
        currentIndex.value >= 0 ? playlist.value[currentIndex.value] : null
    )

    const progress = computed(() =>
        duration.value > 0 ? currentTime.value / duration.value : 0
    )

    const playState = usePlayState(
        audio,
        playlist,
        currentIndex,
        currentTime,
        (track, coverUrl) => {
            taskbarIcon.update(coverUrl)
            mediaSession.update(track)
        }
    )

    useAccentColor(currentTrack, settingsStore)

    audio.addEventListener('error', (e) => {
        console.error('Audio error:', e, 'src:', audio.src, 'code:', audio.error?.code)
    })
    audio.addEventListener('timeupdate', () => {
        currentTime.value = audio.currentTime
    })
    audio.addEventListener('loadedmetadata', () => {
        duration.value = audio.duration
    })
    audio.addEventListener('ended', () => {
        if (settingsStore.preventSleep) {
            invoke('allow_sleep').catch(() => {})
        }
        handleTrackEnd()
    })
    let saveTimer: ReturnType<typeof setInterval> | null = null
    audio.addEventListener('play', () => {
        isPlaying.value = true
        if (settingsStore.preventSleep) {
            invoke('prevent_sleep').catch(() => {})
        }
        if (saveTimer) clearInterval(saveTimer)
        saveTimer = setInterval(playState.savePlayState, 5000)
    })
    audio.addEventListener('pause', () => {
        isPlaying.value = false
        if (settingsStore.preventSleep) {
            invoke('allow_sleep').catch(() => {})
        }
        if (saveTimer) { clearInterval(saveTimer); saveTimer = null }
        playState.savePlayState()
    })

    audio.volume = volume.value
    watch(volume, (v) => { audio.volume = v })

    function setPlaylist(files: Track[], startIndex = 0) {
        playlist.value = files
        playTrackAt(startIndex)
    }

    function playTrackAt(index: number) {
        if (index < 0 || index >= playlist.value.length) return
        audioGraph.init(audio, eq, effects)
        playState.clearRestoreListener()

        const wasPlaying = isPlaying.value && !!audio.src
        const startNewTrack = () => {
            currentIndex.value = index
            const track = playlist.value[index]
            audio.src = convertFileSrc(track.path)
            audio.play().catch(e => console.error('play() failed:', e, 'src:', audio.src))
            playState.savePlayState()
            audioGraph.applyFadeIn()

            if (track.coverUrl) {
                taskbarIcon.update(track.coverUrl)
                mediaSession.update(track)
            } else {
                invoke<string | null>('read_cover', { path: track.path }).then(cover => {
                    if (cover && playlist.value[index]) {
                        playlist.value[index].coverUrl = cover
                        taskbarIcon.update(cover)
                        mediaSession.update(playlist.value[index])
                    }
                }).catch(() => {})
            }
        }

        audioGraph.transitionToNewTrack(wasPlaying, startNewTrack)
    }

    function togglePlay() {
        if (!audio.src) return
        if (audio.paused) audio.play().catch(e => console.error('play() failed:', e, 'src:', audio.src))
        else audio.pause()
    }

    function stop() {
        audio.pause()
        audio.currentTime = 0
        isPlaying.value = false
    }

    function next() {
        const nextIdx = pickNextIndex(currentIndex.value, playlist.value.length, shuffle.value)
        if (nextIdx >= 0) playTrackAt(nextIdx)
    }

    function prev() {
        const prevIdx = pickPrevIndex(currentIndex.value, playlist.value.length, shuffle.value)
        if (prevIdx >= 0) playTrackAt(prevIdx)
    }

    function seek(fraction: number) {
        if (duration.value > 0) {
            audio.currentTime = fraction * duration.value
            currentTime.value = audio.currentTime
        }
    }

    function setVolume(v: number) {
        volume.value = Math.max(0, Math.min(1, v))
    }

    function toggleShuffle() {
        shuffle.value = !shuffle.value
    }

    function cycleLoopMode() {
        const modes = [LoopMode.None, LoopMode.RepeatOne, LoopMode.RepeatAll]
        const idx = modes.indexOf(loopMode.value)
        loopMode.value = modes[(idx + 1) % modes.length]
    }

    function handleTrackEnd() {
        if (loopMode.value === LoopMode.RepeatOne) {
            audio.currentTime = 0
            audio.play().catch(e => console.error('play() failed:', e, 'src:', audio.src))
        } else if (loopMode.value === LoopMode.RepeatAll) {
            next()
        } else {
            // None 模式：shuffle 下持续随机播放（等价 RepeatAll 的随机版），
            // 顺序播放时播完最后一首停止
            if (shuffle.value) {
                next()
            } else if (currentIndex.value < playlist.value.length - 1) {
                next()
            } else {
                isPlaying.value = false
            }
        }
    }

    mediaSession.setupActionHandlers(togglePlay, prev, next)

    playState.restorePlayState().finally(() => { playState.isRestoringState.value = false })

    function getAnalyser(): AnalyserNode {
        return audioGraph.getAnalyser()
    }

    function setEqGain(band: number, gain: number) {
        eq.setEqGain(band, gain)
    }

    function setEqPreset(name: string, gains: number[]) {
        eq.setEqPreset(name, gains)
    }

    function toggleEq() {
        eq.toggleEq()
    }

    function resetEq() {
        eq.resetEq()
    }

    function appendTracks(files: Track[]) {
        const existing = new Set(playlist.value.map(f => f.path))
        const newFiles = files.filter(f => !existing.has(f.path))
        if (newFiles.length > 0) {
            playlist.value = [...playlist.value, ...newFiles]
        }
    }

    return {
        playlist, currentIndex, isPlaying, isRestoringState: playState.isRestoringState,
        currentTime, duration, volume, shuffle, loopMode,
        currentTrack, progress,
        eqGains: eq.eqGains, eqEnabled: eq.eqEnabled, EQ_FREQUENCIES, eqPreset: eq.eqPreset,
        playbackSpeed: effects.playbackSpeed, stereoBalance: effects.stereoBalance,
        reverbMix: effects.reverbMix, bassBoost: effects.bassBoost, vocalBoost: effects.vocalBoost,
        setPlaylist, appendTracks, playTrackAt, togglePlay, stop, next, prev, seek, setVolume,
        toggleShuffle, cycleLoopMode, getAnalyser, setEqGain, toggleEq, resetEq, setEqPreset,
        setPlaybackSpeed: effects.setPlaybackSpeed, setStereoBalance: effects.setStereoBalance,
        setReverbMix: effects.setReverbMix, setBassBoost: effects.setBassBoost,
        setVocalBoost: effects.setVocalBoost, resetEffects: effects.reset,
        playerViewStyle,
    }
})
