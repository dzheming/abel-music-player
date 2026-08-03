import { invoke } from '@tauri-apps/api/core'

export function useTaskbarIcon() {
    function update(coverUrl?: string) {
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

    return { update }
}
