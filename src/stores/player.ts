import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { LoopMode } from '../types'
import { useEqualizer, EQ_FREQUENCIES } from '../composables/useEqualizer'
import { useAudioEffects } from '../composables/useAudioEffects'
import { useSettingsStore } from './settings'
import { stripExtension } from '../utils/format'
import { adjustBrightness } from '../utils/color'
import { extractDominantColor } from '../utils/extract-color'
import { toTrack } from '../types'
import type { Track, RawTrack } from '../types'

interface SavedPlayState {
    paths: string[]
    currentIndex: number
    currentTime: number
}

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

    invoke('get_setting', { key: 'volume' }).then(v => {
        if (v) volume.value = Number(v)
    }).catch(() => {})
    invoke('get_setting', { key: 'shuffle' }).then(v => {
        if (v) shuffle.value = v === 'true'
    }).catch(() => {})
    invoke('get_setting', { key: 'loop-mode' }).then(v => {
        if (v && Object.values(LoopMode).includes(v as LoopMode)) loopMode.value = v as LoopMode
    }).catch(() => {})

    const isRestoringState = ref(true)

    const eq = useEqualizer()
    const effects = useAudioEffects()

    const currentTrack = computed(() =>
        currentIndex.value >= 0 ? playlist.value[currentIndex.value] : null
    )

    const progress = computed(() =>
        duration.value > 0 ? currentTime.value / duration.value : 0
    )

    audio.addEventListener('timeupdate', () => {
        currentTime.value = audio.currentTime
    })
    audio.addEventListener('loadedmetadata', () => {
        duration.value = audio.duration
    })
    audio.addEventListener('ended', () => {
        if (useSettingsStore().preventSleep) {
            invoke('allow_sleep').catch(() => {})
        }
        handleTrackEnd()
    })
    let saveTimer: ReturnType<typeof setInterval> | null = null
    audio.addEventListener('play', () => {
        isPlaying.value = true
        if (useSettingsStore().preventSleep) {
            invoke('prevent_sleep').catch(() => {})
        }
        if (saveTimer) clearInterval(saveTimer)
        saveTimer = setInterval(savePlayState, 5000)
    })
    audio.addEventListener('pause', () => {
        isPlaying.value = false
        if (useSettingsStore().preventSleep) {
            invoke('allow_sleep').catch(() => {})
        }
        if (saveTimer) { clearInterval(saveTimer); saveTimer = null }
        savePlayState()
    })

    audio.volume = volume.value
    let volumeSaveTimer: ReturnType<typeof setTimeout> | null = null
    watch(volume, (v) => {
        audio.volume = v
        if (volumeSaveTimer) clearTimeout(volumeSaveTimer)
        volumeSaveTimer = setTimeout(() => {
            invoke('set_setting', { key: 'volume', value: String(v) }).catch(() => {})
        }, 1000)
    })
    watch(shuffle, (v) => {
        invoke('set_setting', { key: 'shuffle', value: String(v) }).catch(() => {})
    })
    watch(loopMode, (m) => {
        invoke('set_setting', { key: 'loop-mode', value: m }).catch(() => {})
    })

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
                    audio.currentTime = state.currentTime
                    currentTime.value = state.currentTime
                    invoke<string | null>('read_cover', { path: track.path }).then(cover => {
                        if (cover && playlist.value[state.currentIndex]) {
                            playlist.value[state.currentIndex].coverUrl = cover
                            updateTaskbarIcon(cover)
                            updateMediaSession(playlist.value[state.currentIndex])
                        }
                    }).catch(() => {})
                }
            }
        } catch (e) {
            console.error('Failed to restore play state:', e)
        }
    }

    function setPlaylist(files: Track[], startIndex = 0) {
        playlist.value = files
        playTrackAt(startIndex)
    }

    let fadeOutTimer: ReturnType<typeof setTimeout> | null = null

    function playTrackAt(index: number) {
        if (index < 0 || index >= playlist.value.length) return
        initAudioContext()

        if (fadeOutTimer) { clearTimeout(fadeOutTimer); fadeOutTimer = null }

        const wasPlaying = isPlaying.value && audio.src
        const startNewTrack = () => {
            currentIndex.value = index
            const track = playlist.value[index]
            audio.src = convertFileSrc(track.path)
            audio.play()
            savePlayState()

            if (fadeGainNode && audioContext) {
                fadeGainNode.gain.cancelScheduledValues(audioContext.currentTime)
                fadeGainNode.gain.setValueAtTime(0, audioContext.currentTime)
                fadeGainNode.gain.linearRampToValueAtTime(1, audioContext.currentTime + FADE_IN_DURATION)
            }

            if (track.coverUrl) {
                updateTaskbarIcon(track.coverUrl)
                updateMediaSession(track)
            } else {
                invoke<string | null>('read_cover', { path: track.path }).then(cover => {
                    if (cover && playlist.value[index]) {
                        playlist.value[index].coverUrl = cover
                        updateTaskbarIcon(cover)
                        updateMediaSession(playlist.value[index])
                    }
                }).catch(() => {})
            }
        }

        if (wasPlaying && fadeGainNode && audioContext) {
            fadeGainNode.gain.cancelScheduledValues(audioContext.currentTime)
            fadeGainNode.gain.setValueAtTime(fadeGainNode.gain.value, audioContext.currentTime)
            fadeGainNode.gain.linearRampToValueAtTime(0, audioContext.currentTime + FADE_OUT_DURATION)
            fadeOutTimer = setTimeout(() => {
                fadeOutTimer = null
                startNewTrack()
            }, FADE_OUT_DURATION * 1000)
        } else {
            startNewTrack()
        }
    }

    function updateMediaSession(track: Track) {
        if ('mediaSession' in navigator) {
            const artwork = track.coverUrl ? [{ src: track.coverUrl }] : []
            navigator.mediaSession.metadata = new MediaMetadata({
                title: track.title || stripExtension(track.fileName),
                artist: track.artist || '',
                album: track.album || '',
                artwork,
            })
        }
    }

    function updateTaskbarIcon(coverUrl?: string) {
        if (coverUrl) {
            invoke('set_taskbar_icon', { iconBase64: coverUrl }).catch((e) => {
                console.error('set_taskbar_icon failed:', e)
            })
        } else {
            invoke('reset_taskbar_icon').catch((e) => {
                console.error('reset_taskbar_icon failed:', e)
            })
        }
    }

    function togglePlay() {
        if (!audio.src) return
        if (audio.paused) audio.play()
        else audio.pause()
    }

    function stop() {
        audio.pause()
        audio.currentTime = 0
        isPlaying.value = false
    }

    function next() {
        if (playlist.value.length === 0) return
        let nextIdx: number
        if (shuffle.value) {
            nextIdx = Math.floor(Math.random() * playlist.value.length)
        } else {
            nextIdx = (currentIndex.value + 1) % playlist.value.length
        }
        playTrackAt(nextIdx)
    }

    function prev() {
        if (playlist.value.length === 0) return
        let prevIdx: number
        if (shuffle.value) {
            prevIdx = Math.floor(Math.random() * playlist.value.length)
        } else {
            prevIdx = (currentIndex.value - 1 + playlist.value.length) % playlist.value.length
        }
        playTrackAt(prevIdx)
    }

    function seek(fraction: number) {
        if (duration.value > 0) {
            audio.currentTime = fraction * duration.value
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
            audio.play()
        } else if (loopMode.value === LoopMode.RepeatAll) {
            next()
        } else {
            if (shuffle.value) {
                next()
            }else if (currentIndex.value < playlist.value.length - 1) {
                next()
            } else {
                isPlaying.value = false
            }
        }
    }

    let mediaKeysEnabled = true
    invoke('get_setting', { key: 'media-keys' }).then(v => {
        if (v) mediaKeysEnabled = v !== 'false'
    }).catch(() => {})

    function isMediaKeysEnabled(): boolean {
        return mediaKeysEnabled
    }
    if ('mediaSession' in navigator) {
        navigator.mediaSession.setActionHandler('play', () => { if (isMediaKeysEnabled()) togglePlay() })
        navigator.mediaSession.setActionHandler('pause', () => { if (isMediaKeysEnabled()) togglePlay() })
        navigator.mediaSession.setActionHandler('previoustrack', () => { if (isMediaKeysEnabled()) prev() })
        navigator.mediaSession.setActionHandler('nexttrack', () => { if (isMediaKeysEnabled()) next() })
    }

    restorePlayState().finally(() => { isRestoringState.value = false })

    let systemAccentColor: string | null = null
    invoke<string>('get_system_accent_color').then(c => { systemAccentColor = c }).catch(() => {})

    watch(() => currentTrack.value?.coverUrl, (coverUrl) => {
        if (coverUrl) {
            extractDominantColor(coverUrl).then(color => {
                if (color) {
                    document.documentElement.style.setProperty('--color-accent', color)
                    document.documentElement.style.setProperty('--color-accent-hover', adjustBrightness(color, -20))
                }
            })
        } else if (systemAccentColor) {
            document.documentElement.style.setProperty('--color-accent', systemAccentColor)
            document.documentElement.style.setProperty('--color-accent-hover', adjustBrightness(systemAccentColor, -20))
        }
    })

    let audioContext: AudioContext | null = null
    let analyser: AnalyserNode | null = null
    let fadeGainNode: GainNode | null = null
    const FADE_OUT_DURATION = 0.3
    const FADE_IN_DURATION = 0.5

    function createImpulseResponse(ctx: AudioContext, decay: number): AudioBuffer {
        const length = Math.floor(ctx.sampleRate * decay)
        const buffer = ctx.createBuffer(2, length, ctx.sampleRate)
        for (let ch = 0; ch < 2; ch++) {
            const data = buffer.getChannelData(ch)
            for (let i = 0; i < length; i++) {
                data[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / length, decay)
            }
        }
        return buffer
    }

    function initAudioContext() {
        if (audioContext) return
        audioContext = new AudioContext()
        analyser = audioContext.createAnalyser()
        analyser.fftSize = 512
        fadeGainNode = audioContext.createGain()

        const source = audioContext.createMediaElementSource(audio)

        const filters = EQ_FREQUENCIES.map((freq, i) => {
            const filter = audioContext!.createBiquadFilter()
            filter.type = i === 0 ? 'lowshelf' : i === EQ_FREQUENCIES.length - 1 ? 'highshelf' : 'peaking'
            filter.frequency.value = freq
            filter.Q.value = 1.4
            filter.gain.value = eq.eqEnabled.value ? eq.eqGains.value[i] : 0
            return filter
        })

        let prev: AudioNode = source
        for (const filter of filters) {
            prev.connect(filter)
            prev = filter
        }

        const bassFilter = audioContext.createBiquadFilter()
        bassFilter.type = 'lowshelf'
        bassFilter.frequency.value = 150
        bassFilter.gain.value = 0

        const vocalFilter = audioContext.createBiquadFilter()
        vocalFilter.type = 'peaking'
        vocalFilter.frequency.value = 2500
        vocalFilter.Q.value = 1.2
        vocalFilter.gain.value = 0

        const stereoPanner = audioContext.createStereoPanner()
        stereoPanner.pan.value = 0

        const convolverNode = audioContext.createConvolver()
        convolverNode.buffer = createImpulseResponse(audioContext, 2)

        const dryGain = audioContext.createGain()
        dryGain.gain.value = 1

        const wetGain = audioContext.createGain()
        wetGain.gain.value = 0

        const reverbMerge = audioContext.createGain()

        prev.connect(bassFilter)
        bassFilter.connect(vocalFilter)
        vocalFilter.connect(stereoPanner)

        stereoPanner.connect(dryGain)
        stereoPanner.connect(convolverNode)
        convolverNode.connect(wetGain)

        dryGain.connect(reverbMerge)
        wetGain.connect(reverbMerge)

        reverbMerge.connect(fadeGainNode)
        fadeGainNode.connect(analyser)
        analyser.connect(audioContext.destination)

        eq.setFilters(filters)
        effects.setAudio(audio)
        effects.setNodes({ stereoPanner, bassFilter, vocalFilter, convolverNode, wetGain, dryGain, audioContext })
    }

    function getAnalyser(): AnalyserNode {
        initAudioContext()
        if (audioContext!.state === 'suspended') {
            audioContext!.resume()
        }
        return analyser!
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
        playlist, currentIndex, isPlaying, isRestoringState, currentTime, duration, volume, shuffle, loopMode,
        currentTrack, progress,
        eqGains: eq.eqGains, eqEnabled: eq.eqEnabled, EQ_FREQUENCIES, eqPreset: eq.eqPreset,
        playbackSpeed: effects.playbackSpeed, stereoBalance: effects.stereoBalance,
        reverbMix: effects.reverbMix, bassBoost: effects.bassBoost, vocalBoost: effects.vocalBoost,
        setPlaylist, appendTracks, playTrackAt, togglePlay, stop, next, prev, seek, setVolume,
        toggleShuffle, cycleLoopMode, getAnalyser, setEqGain, toggleEq, resetEq, setEqPreset,
        setPlaybackSpeed: effects.setPlaybackSpeed, setStereoBalance: effects.setStereoBalance,
        setReverbMix: effects.setReverbMix, setBassBoost: effects.setBassBoost,
        setVocalBoost: effects.setVocalBoost, resetEffects: effects.reset,
    }
})
