import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PlayList, PlayListTrack } from '../types'

export const usePlaylistStore = defineStore('playlist', () => {
    const DEFAULT_PLAYLIST_NAME = '默认列表'
    const playlists = ref<PlayList[]>([])
    const defaultPlaylistId = ref<number | null>(null)
    const currentPlaylistId = ref<number | null>(null)
    const playingPlaylistId = ref<number | null>(null)

    invoke('get_setting', { key: 'playing-playlist-id' }).then(v => {
        if (v) playingPlaylistId.value = JSON.parse(v as string)
    }).catch(() => {})
    const currentTracks = ref<PlayListTrack[]>([])
    const isLoading = ref(false)

    async function loadPlaylists() {
        try {
            playlists.value = await invoke('get_playlists')
        } catch (e) {
            console.error('Failed to load playlists:', e)
        }
    }

    async function createPlaylist(name: string) {
        try {
            const playlist: PlayList = await invoke('create_playlist', { name })
            playlists.value.unshift(playlist)
            return playlist
        } catch (e) {
            console.error('Failed to create playlist:', e)
            return null
        }
    }

    async function deletePlaylist(id: number) {
        try {
            await invoke('delete_playlist', { id })
            playlists.value = playlists.value.filter(p => p.id !== id)
            if (currentPlaylistId.value === id) {
                currentPlaylistId.value = null
                currentTracks.value = []
            }
            if (playingPlaylistId.value === id) {
                clearPlayingState()
            }
        } catch (e) {
            console.error('Failed to delete playlist:', e)
        }
    }

    async function renamePlaylist(id: number, name: string) {
        try {
            await invoke('rename_playlist', { id, name })
            const p = playlists.value.find(p => p.id === id)
            if (p) p.name = name
        } catch (e) {
            console.error('Failed to rename playlist:', e)
        }
    }

    async function selectPlaylist(id: number) {
        currentPlaylistId.value = id
        isLoading.value = true
        try {
            currentTracks.value = await invoke('get_playlist_tracks', { playlistId: id })
        } catch (e) {
            console.error('Failed to load playlist tracks:', e)
            currentTracks.value = []
        } finally {
            isLoading.value = false
        }
    }

    async function addToPlaylist(playlistId: number, paths: string[]) {
        try {
            const added: number = await invoke('add_to_playlist', { playlistId, paths })
            const p = playlists.value.find(p => p.id === playlistId)
            if (p) p.track_count += added
            if (currentPlaylistId.value === playlistId && added > 0) {
                await selectPlaylist(playlistId)
            }
        } catch (e) {
            console.error('Failed to add to playlist:', e)
        }
    }

    async function removeFromPlaylist(playlistId: number, paths: string[]) {
        try {
            const removed: number = await invoke('remove_from_playlist', { playlistId, paths })
            const p = playlists.value.find(p => p.id === playlistId)
            if (p) p.track_count = Math.max(0, p.track_count - removed)
            if (currentPlaylistId.value === playlistId) {
                currentTracks.value = currentTracks.value.filter(t => !paths.includes(t.path))
            }
        } catch (e) {
            console.error('Failed to remove from playlist:', e)
        }
    }

    async function clearPlayList(playlistId: number) {
        try {
            await invoke('clear_playlist', { playlistId })
            const p = playlists.value.find(p => p.id === playlistId)
            if (p) p.track_count = 0
            if (currentPlaylistId.value === playlistId) {
                currentTracks.value = []
            }
            if (playingPlaylistId.value === playlistId) {
                clearPlayingState()
            }
        } catch (e) {
            console.error('Failed to clear playlist:', e)
        }
    }

    function clearPlayingState() {
        playingPlaylistId.value = null
        invoke('set_setting', { key: 'playing-playlist-id', value: 'null' }).catch(() => {})
    }

    async function ensureDefaultPlaylist() {
        const existing = playlists.value.find(p => p.name === DEFAULT_PLAYLIST_NAME)
        if (existing) {
            defaultPlaylistId.value = existing.id
        } else {
            const pl = await createPlaylist(DEFAULT_PLAYLIST_NAME)
            if (pl) defaultPlaylistId.value = pl.id
        }
    }

    async function addToDefault(paths: string[]) {
        if (!defaultPlaylistId.value) await ensureDefaultPlaylist()
        if (defaultPlaylistId.value) {
            await addToPlaylist(defaultPlaylistId.value, paths)
        }
    }

    async function init() {
        await loadPlaylists()
        await ensureDefaultPlaylist()
    }

    init()

    return {
        playlists, defaultPlaylistId, currentPlaylistId, playingPlaylistId, currentTracks, isLoading,
        loadPlaylists, createPlaylist,  deletePlaylist, renamePlaylist,
        selectPlaylist, addToPlaylist, removeFromPlaylist, clearPlayList, addToDefault, ensureDefaultPlaylist,
    }
})