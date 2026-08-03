import { watch } from 'vue'
import type { Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { LoopMode } from '../types'

interface PersistedSettings {
    volume: Ref<number>
    shuffle: Ref<boolean>
    loopMode: Ref<LoopMode>
    playerViewStyle: Ref<string>
}

/** 加载持久化设置到响应式 ref，并 watch 变化写回 */
export function useSettingsPersistence(settings: PersistedSettings) {
    const { volume, shuffle, loopMode, playerViewStyle } = settings

    // 加载
    invoke('get_setting', { key: 'volume' }).then(v => {
        if (v) volume.value = Number(v)
    }).catch(() => {})
    invoke('get_setting', { key: 'shuffle' }).then(v => {
        if (v) shuffle.value = v === 'true'
    }).catch(() => {})
    invoke('get_setting', { key: 'loop-mode' }).then(v => {
        if (v && Object.values(LoopMode).includes(v as LoopMode)) loopMode.value = v as LoopMode
    }).catch(() => {})
    invoke('get_setting', { key: 'player-view-style' }).then(v => {
        if (v) playerViewStyle.value = v as string
    }).catch(() => {})

    // 持久化
    let volumeSaveTimer: ReturnType<typeof setTimeout> | null = null
    watch(volume, (v) => {
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
    watch(playerViewStyle, (v) => {
        invoke('set_setting', { key: 'player-view-style', value: v }).catch(() => {})
    })
}
