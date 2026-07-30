import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { PlayList, RawPlaylistTrack, Track } from '../types'
import { toTrack } from '../types'

export const usePlaylistStore = defineStore('playlist', () => {
    const DEFAULT_PLAYLIST_NAME = '默认列表'
    const playlists = ref<PlayList[]>([])
    const defaultPlaylistId = ref<number | null>(null)
    const currentPlaylistId = ref<number | null>(null)
    const playingPlaylistId = ref<number | null>(null)

    const currentTracks = ref<Track[]>([])
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
            const raw = await invoke<RawPlaylistTrack[]>('get_playlist_tracks', { playlistId: id })
            currentTracks.value = raw.map(toTrack)
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

    function persistPlayingPlaylistId(id: number | null) {
        const value = id === null ? 'null' : JSON.stringify(id)
        invoke('set_setting', { key: 'playing-playlist-id', value }).catch(e => {
            console.error('Failed to persist playing-playlist-id:', e)
        })
    }

    function setPlayingPlaylist(id: number | null) {
        playingPlaylistId.value = id
        persistPlayingPlaylistId(id)
    }

    function clearPlayingState() {
        setPlayingPlaylist(null)
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
            setPlayingPlaylist(defaultPlaylistId.value)
            if (currentPlaylistId.value !== defaultPlaylistId.value) {
                await selectPlaylist(defaultPlaylistId.value)
            }
        }
    }

    async function init() {
        await loadPlaylists()
        await ensureDefaultPlaylist()
        try {
            const v = await invoke('get_setting', { key: 'playing-playlist-id' })
            if (v) playingPlaylistId.value = JSON.parse(v as string)
        } catch (e) {
            console.error('Failed to restore playing playlist id:', e)
        }
        if (playingPlaylistId.value && playlists.value.some(p => p.id === playingPlaylistId.value)) {
            await selectPlaylist(playingPlaylistId.value)
        }
    }

    return {
        playlists, defaultPlaylistId, currentPlaylistId, playingPlaylistId, currentTracks, isLoading,
        loadPlaylists, createPlaylist, deletePlaylist, renamePlaylist,
        selectPlaylist, addToPlaylist, removeFromPlaylist, clearPlayList, addToDefault, ensureDefaultPlaylist,
        setPlayingPlaylist, clearPlayingState, init,
    }
})
