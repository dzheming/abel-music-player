import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { toTrack } from '../types'
import type { ArtistGroup, AlbumGroup, Track, RawTrack } from '../types'

export const useBrowseStore = defineStore('browse', () => {
    const artists = ref<ArtistGroup[]>([])
    const albums = ref<AlbumGroup[]>([])
    const currentArtist = ref<string | null>(null)
    const currentAlbum = ref<string | null>(null)
    const tracks = ref<Track[]>([])
    const isLoading = ref(false)
    const viewMode = ref<'artists' | 'albums'>('artists')

    async function loadArtists() {
        try {
            artists.value = await invoke('get_artists')
        } catch (e) {
            console.error('Failed to load artists:', e)
        }
    }

    async function loadAlbums() {
        try {
            albums.value = await invoke('get_albums')
        } catch (e) {
            console.error('Failed to load albums:', e)
        }
    }

    async function selectArtist(artist: string) {
        currentArtist.value = artist
        currentAlbum.value = null
        isLoading.value = true
        try {
            const raw: RawTrack[] = await invoke('get_tracks_by_artist', { artist })
            tracks.value = raw.map(toTrack)
        } catch (e) {
            console.error('Failed to load artist tracks:', e)
            tracks.value = []
        } finally {
            isLoading.value = false
        }
    }

    async function selectAlbum(album: string) {
        currentAlbum.value = album
        currentArtist.value = null
        isLoading.value = true
        try {
            const raw: RawTrack[] = await invoke('get_tracks_by_album', { album })
            tracks.value = raw.map(toTrack)
        } catch (e) {
            console.error('Failed to load album tracks:', e)
            tracks.value = []
        } finally {
            isLoading.value = false
        }
    }

    function clearSelection() {
        currentArtist.value = null
        currentAlbum.value = null
        tracks.value = []
    }

    async function refresh() {
        await loadArtists()
        await loadAlbums()
    }

    return {
        artists, albums, currentArtist, currentAlbum, tracks, isLoading, viewMode,
        loadArtists, loadAlbums, selectArtist, selectAlbum, clearSelection, refresh,
    }
})
