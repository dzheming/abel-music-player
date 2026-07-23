import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ThemeMode } from '../types'

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
    const viewMode = ref<'list' | 'card'>('list')

    invoke('get_setting', { key: 'close-to-tray' }).then(v => {
        if (v) closeToTray.value = v !== 'false'
    }).catch(() => {})
    invoke('get_setting', { key: 'media-keys' }).then(v => {
        if (v) mediaKeysEnabled.value = v !== 'false'
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

    function adjustBrightness(hex: string, amount: number): string {
        const num = parseInt(hex.replace('#', ''), 16)
        let r = Math.min(255, Math.max(0, ((num >> 16) & 0xff) + amount))
        let g = Math.min(255, Math.max(0, ((num >> 8) & 0xff) + amount))
        let b = Math.min(255, Math.max(0, (num & 0xff) + amount))
        return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`
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

    function toggleViewMode() {
        viewMode.value = viewMode.value === 'list' ? 'card' : 'list'
        invoke('set_setting', { key: 'view-mode', value: viewMode.value }).catch(() => {})
    }

    watch(theme, applyTheme, { immediate: true })
    loadSystemAccentColor()

    return { 
        theme, accentColor, closeToTray, mediaKeysEnabled, viewMode,
        toggleTheme, toggleCloseToTray, toggleMediaKeys, toggleViewMode, loadSystemAccentColor,
    }
})