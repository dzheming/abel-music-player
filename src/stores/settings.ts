import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ThemeMode } from '../types'
import { adjustBrightness } from '../utils/color'

export const useSettingsStore = defineStore('settings', () => {
    function getSystemTheme(): ThemeMode {
        return window.matchMedia('(prefers-color-scheme: dark)').matches
            ? ThemeMode.Dark
            : ThemeMode.Light
    }

    const theme = ref<ThemeMode>(getSystemTheme())
    const accentColor = ref('#007aff')
    const closeToTray = ref(true)
    const mediaKeysEnabled = ref(true)
    const preventSleep = ref(false)
    const viewMode = ref<'list' | 'card'>('list')

    invoke('get_setting', { key: 'close-to-tray' }).then(v => {
        if (v) closeToTray.value = v !== 'false'
    }).catch(() => {})
    invoke('get_setting', { key: 'media-keys' }).then(v => {
        if (v) mediaKeysEnabled.value = v !== 'false'
    }).catch(() => {})
    invoke('get_setting', { key: 'prevent-sleep' }).then(v => {
        if (v) preventSleep.value = v === 'true'
    }).catch(() => {})
    invoke('get_setting', { key: 'view-mode' }).then(v => {
        if (v) viewMode.value = v as 'list' | 'card'
    }).catch(() => {})

    function applyTheme() {
        document.documentElement.setAttribute('data-theme', theme.value)
    }

    function applyAccentColor(color: string) {
        document.documentElement.style.setProperty('--color-accent', color)
        document.documentElement.style.setProperty('--color-accent-hover', adjustBrightness(color, -20))
    }

    function toggleTheme() {
        theme.value = theme.value === ThemeMode.Dark ? ThemeMode.Light : ThemeMode.Dark
    }

    async function loadSystemAccentColor() {
        try { 
            const color: string = await invoke('get_system_accent_color')
            accentColor.value = color
            applyAccentColor(color)
        } catch (e) {
            console.error('Failed to get system accent color:', e)
        }
    }

    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        theme.value = e.matches ? ThemeMode.Dark : ThemeMode.Light
    })

    function toggleCloseToTray() {
        closeToTray.value = !closeToTray.value
        invoke('set_setting', { key: 'close-to-tray', value: String(closeToTray.value) }).catch(() => {})
    }

    function toggleMediaKeys() {
        mediaKeysEnabled.value = !mediaKeysEnabled.value
        invoke('set_setting', { key: 'media-keys', value: String(mediaKeysEnabled.value) }).catch(() => {})
    }

    function togglePreventSleep() {
        preventSleep.value = !preventSleep.value
        invoke('set_setting', { key: 'prevent-sleep', value: String(preventSleep.value) }).catch(() => {})
    }

    function toggleViewMode() {
        viewMode.value = viewMode.value === 'list' ? 'card' : 'list'
        invoke('set_setting', { key: 'view-mode', value: viewMode.value }).catch(() => {})
    }

    watch(theme, applyTheme, { immediate: true })
    loadSystemAccentColor()

    return {
        theme, accentColor, closeToTray, mediaKeysEnabled, preventSleep, viewMode,
        toggleTheme, toggleCloseToTray, toggleMediaKeys, togglePreventSleep, toggleViewMode, loadSystemAccentColor, applyAccentColor,
    }
})