import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AudioEffectNodes {
    stereoPanner: StereoPannerNode
    bassFilter: BiquadFilterNode
    vocalFilter: BiquadFilterNode
    convolverNode: ConvolverNode
    wetGain: GainNode
    dryGain: GainNode
    audioContext: AudioContext
}

export function useAudioEffects() {
    const playbackSpeed = ref(1.0)
    const stereoBalance = ref(0)
    const reverbMix = ref(0)
    const bassBoost = ref(0)
    const vocalBoost = ref(0)

    let nodes: AudioEffectNodes | null = null
    let audio: HTMLAudioElement | null = null

    invoke('get_setting', { key: 'audio-effects' }).then(v => {
        if (v) {
            const saved = JSON.parse(v as string)
            if (saved && typeof saved === 'object') {
                if (typeof saved.playbackSpeed === 'number') playbackSpeed.value = saved.playbackSpeed
                if (typeof saved.stereoBalance === 'number') stereoBalance.value = saved.stereoBalance
                if (typeof saved.reverbMix === 'number') reverbMix.value = saved.reverbMix
                if (typeof saved.bassBoost === 'number') bassBoost.value = saved.bassBoost
                if (typeof saved.vocalBoost === 'number') vocalBoost.value = saved.vocalBoost
                applyAll()
            }
        }
    }).catch(() => {})

    let saveTimer: ReturnType<typeof setTimeout> | null = null
    function save() {
        if (saveTimer) clearTimeout(saveTimer)
        saveTimer = setTimeout(() => {
            invoke('set_setting', {
                key: 'audio-effects',
                value: JSON.stringify({
                    playbackSpeed: playbackSpeed.value,
                    stereoBalance: stereoBalance.value,
                    reverbMix: reverbMix.value,
                    bassBoost: bassBoost.value,
                    vocalBoost: vocalBoost.value,
                })
            }).catch(() => {})
        }, 1000)
    }
    
    function setAudio(el: HTMLAudioElement) {
        audio = el
        audio.playbackRate = playbackSpeed.value
    }

    function setNodes(n: AudioEffectNodes) {
        nodes = n
        applyAll()
    }

    function applyAll() {
        if (audio) audio.playbackRate = playbackSpeed.value
        if (!nodes) return
        nodes.stereoPanner.pan.value = stereoBalance.value / 100
        nodes.bassFilter.gain.value = bassBoost.value * 0.15
        nodes.vocalFilter.gain.value = vocalBoost.value * 0.12
        applyReverbMix()
    }

    function applyReverbMix() {
        if (!nodes) return
        const wet = reverbMix.value / 100
        nodes.wetGain.gain.value = wet
        nodes.dryGain.gain.value = 1 - wet * 0.5
    }

    function setPlaybackSpeed(v: number) {
        playbackSpeed.value = v
        if (audio) audio.playbackRate = v
        save()
    }

    function setStereoBalance(v: number) {
        stereoBalance.value = v
        if (nodes) nodes.stereoPanner.pan.value = v / 100
        save()
    }

    function setReverbMix(v: number) {
        reverbMix.value = v
        applyReverbMix()
        save()
    }

    function setBassBoost(v: number) {
        bassBoost.value = v
        if (nodes) nodes.bassFilter.gain.value = v * 0.15
        save()
    }

    function setVocalBoost(v: number) {
        vocalBoost.value = v
        if (nodes) nodes.vocalFilter.gain.value = v * 0.12
        save()
    }

    function reset() {
        playbackSpeed.value = 1.0
        stereoBalance.value = 0
        reverbMix.value = 0
        bassBoost.value = 0
        vocalBoost.value = 0
        applyAll()
        save()
    }

    return {
        playbackSpeed, stereoBalance, reverbMix, bassBoost, vocalBoost,
        setAudio, setNodes, setPlaybackSpeed, setStereoBalance,
        setReverbMix, setBassBoost, setVocalBoost, reset,
    }
}