import { EQ_FREQUENCIES, useEqualizer } from './useEqualizer'
import { useAudioEffects } from './useAudioEffects'

const FADE_OUT_DURATION = 0.3
const FADE_IN_DURATION = 0.5

export function useAudioGraph() {
    let audioContext: AudioContext | null = null
    let analyser: AnalyserNode | null = null
    let fadeGainNode: GainNode | null = null
    let fadeOutTimer: ReturnType<typeof setTimeout> | null = null

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

    function init(
        audio: HTMLAudioElement,
        eq: ReturnType<typeof useEqualizer>,
        effects: ReturnType<typeof useAudioEffects>
    ) {
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
        if (!audioContext || !analyser) {
            throw new Error('AudioContext not initialized')
        }
        if (audioContext.state === 'suspended') {
            audioContext.resume()
        }
        return analyser
    }

    function applyFadeIn() {
        if (!fadeGainNode || !audioContext) return
        fadeGainNode.gain.cancelScheduledValues(audioContext.currentTime)
        fadeGainNode.gain.setValueAtTime(0, audioContext.currentTime)
        fadeGainNode.gain.linearRampToValueAtTime(1, audioContext.currentTime + FADE_IN_DURATION)
    }

    /** 切歌时的淡入淡出过渡，内部管理 fadeOutTimer */
    function transitionToNewTrack(wasPlaying: boolean, startNewTrack: () => void) {
        if (fadeOutTimer) { clearTimeout(fadeOutTimer); fadeOutTimer = null }

        if (wasPlaying && audioContext && fadeGainNode) {
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

    return { init, getAnalyser, applyFadeIn, transitionToNewTrack }
}
