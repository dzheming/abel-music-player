import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const EQ_FREQUENCIES = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000]

export const EQ_PRESETS: { name: string; gains: number[] }[] = [
    { name: '默认', gains: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0] },
    { name: '流行', gains: [-1, 4, 6, 7, 5, 0, -2, -2, -1, -1] },
    { name: '摇滚', gains: [5, 4, 3, 1, -1, -1, 0, 2, 3, 4] },
    { name: '古典', gains: [0, 0, 0, 0, 0, 0, -3, -3, -3, -5] },
    { name: '爵士', gains: [0, 0, 1, 4, 4, 4, 0, 1, 3, 3] },
    { name: '电子', gains: [4, 3, 1, 0, -2, 2, 0, 1, 4, 5] },
    { name: '人声', gains: [-2, -3, -3, 1, 4, 4, 3, 1, 0, -2] },
    { name: '低音', gains: [6, 5, 4, 2, 0, 0, 0, 0, 0, 0] },
    { name: '高音', gains: [0, 0, 0, 0, 0, 1, 2, 4, 5, 6] },
]

export function useEqualizer() {
    const eqGains = ref<number[]>(EQ_FREQUENCIES.map(() => 0))
    const eqEnabled = ref(true)
    const eqPreset = ref('默认')

    invoke('get_setting', { key: 'eq-gains' }).then(v => {
        if (v) eqGains.value = JSON.parse(v as string)
    }).catch(() => {})
    invoke('get_setting', { key: 'eq-enabled' }).then(v => {
        if (v) eqEnabled.value = v !== 'false'
    }).catch(() => {})
    invoke('get_setting', { key: 'eq-preset' }).then(v => {
        if (v) eqPreset.value = v as string
    }).catch(() => {})

    let eqFilters: BiquadFilterNode[] = []

    function setFilters(filters: BiquadFilterNode[]) {
        eqFilters = filters
        eqFilters.forEach((filter, i) => {
            filter.gain.value = eqEnabled.value ? eqGains.value[i] : 0
        })
    }

    let eqSaveTimer: ReturnType<typeof setTimeout> | null = null

    function saveEqGains() {
        if (eqSaveTimer) clearTimeout(eqSaveTimer)
        eqSaveTimer = setTimeout(() => {
            invoke('set_setting', { key: 'eq-gains', value: JSON.stringify(eqGains.value) }).catch(() => {})
        }, 1000)
    }

    function setEqGain(band: number, gain: number) {
        eqGains.value[band] = gain
        if (eqFilters[band] && eqEnabled.value) {
            eqFilters[band].gain.value = gain
        }
        saveEqGains()
        eqPreset.value = '自定义'
        invoke('set_setting', { key: 'eq-preset', value: '自定义' }).catch(() => {})
    }

    function setEqPreset(name: string, gains: number[]) {
        eqPreset.value = name
        invoke('set_setting', { key: 'eq-preset', value: name }).catch(() => {})
        gains.forEach((g, i) => {
            eqGains.value[i] = g
            if (eqFilters[i] && eqEnabled.value) {
                eqFilters[i].gain.value = g
            }
        })
        saveEqGains()
    }

    function toggleEq() {
        eqEnabled.value = !eqEnabled.value
        invoke('set_setting', { key: 'eq-enabled', value: String(eqEnabled.value) }).catch(() => {})
        eqFilters.forEach((filter, i) => {
            filter.gain.value = eqEnabled.value ? eqGains.value[i] : 0
        })
    }

    function resetEq() {
        eqGains.value = EQ_FREQUENCIES.map(() => 0)
        eqFilters.forEach(filter => { filter.gain.value = 0 })
        saveEqGains()
        eqPreset.value = '默认'
        invoke('set_setting', { key: 'eq-preset', value: '默认' }).catch(() => {})
    }

    return { 
        eqGains, eqEnabled, eqPreset,
        setFilters, setEqGain, setEqPreset, toggleEq, resetEq,
    }
}