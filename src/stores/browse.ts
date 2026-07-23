import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ArtistGroup, AlbumGroup, AudioFile, RawMetadata } from '../types'

export const useBrowseStore = defineStore('browse', () => {
    const artists = ref<ArtistGroup[]>([])
    const albums = ref<AlbumGroup[]>([])
    const currentArtist = ref<string | null>(null)
    const currentAlbum = ref<string | null>(null)
    const tracks = ref<AudioFile[]>([])
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
            const raw: RawMetadata[] = await invoke('get_tracks_by_artist', { artist })
            tracks.value = raw.map(t => ({
                path: t.path,
                fileName: t.file_name,
                title: t.title || undefined,
                artist: t.artist || undefined,
                album: t.album || undefined,
                duration: t.duration,
                trackNumber: t.track_number || undefined,
            }))
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
            const raw: RawMetadata[] = await invoke('get_tracks_by_album', { album })
            tracks.value = raw.map(t => ({
                path: t.path,
                fileName: t.file_name,
                title: t.title || undefined,
                artist: t.artist || undefined,
                album: t.album || undefined,
                duration: t.duration,
                trackNumber: t.track_number || undefined,
            }))
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

    return {
        artists, albums, currentArtist, currentAlbum, tracks, isLoading, viewMode,
        loadArtists, loadAlbums, selectArtist, selectAlbum, clearSelection,
    }
})