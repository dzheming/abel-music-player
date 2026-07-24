import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import type { AudioFile, MusicFolder, FolderNode, RawMetadata } from '../types'
import { event } from '@tauri-apps/api'

export const useLibraryStore = defineStore('library', () => {
    const folders = ref<MusicFolder[]>([])
    const folderTrees = ref<Map<string, FolderNode>>(new Map())
    const selectedFolderPath = ref<string | null>(null)
    invoke('get_setting', {key: 'music-folders' }).then(v => {
        if (v) folders.value = JSON.parse(v as string)
        for (const folder of folders.value) {
            loadFolderTree(folder.path)
        }
    }).catch(() => {})
    
    invoke('get_setting', { key: 'selected-folder' }).then(v => {
        if (v) {
            selectedFolderPath.value = v as string
            loadFromCache(v as string)
        }
    }).catch(() => {})
    const audioFiles = ref<AudioFile[]>([])
    const isScanning = ref(false)
    const scanProgress = ref('')
    const globalSearchQuery = ref('')
    const globalSearchResults = ref<AudioFile[]>([])
    const isGlobalSearching = ref(false)
    let scanGeneration = 0

    const selectedFolder = computed(() => 
        folders.value.find(f => f.path === selectedFolderPath.value) || null
    )

    async function addFolder() {
        const selected = await open({ directory: true, multiple: false })
        if (!selected) return

        const path = selected as string
        if (folders.value.some(f => f.path === path)) return

        const name = path.split(/[/\\]/).pop() || path
        folders.value.push({ path, name })
        invoke('set_setting', { key: 'music-folders', value: JSON.stringify(folders.value) }).catch(() => {})

        await loadFolderTree(path)
        await scanAndSelect(path)
    }

    function removeFolder(path: string) {
        folders.value = folders.value.filter(f => f.path !== path)
        folderTrees.value.delete(path)
        invoke('set_setting', { key: 'music-folders', value: JSON.stringify(folders.value) }).catch(() => {})
        if (selectedFolderPath.value?.startsWith(path)) {
            selectedFolderPath.value = null
            invoke('set_setting', { key: 'selected-folder', value: '' }).catch(() => {})
            audioFiles.value = []
        }
    }

    async function loadFolderTree(path: string) {
        try {
            const tree: FolderNode = await invoke('scan_folder_tree', { path })
            folderTrees.value.set(path, tree)
        } catch (e) {
            console.error('Failed to load folder tree:', e)
        }
    }

    async function selectFolder(path: string) {
        selectedFolderPath.value = path
        invoke('set_setting', { key: 'selected-folder', value: path }).catch(() => {})
        await loadFromCache(path)
    }

    interface CachedTrackData {
        path: string
        file_name: string
        title: string | null
        artist: string | null
        album: string | null
        duration: number
        track_number: number | null
    }

    async function loadFromCache(path: string) {
        const myGen = ++scanGeneration
        isScanning.value = true
        scanProgress.value = '加载中...'
        try {
            const files: string[] = await invoke('scan_music_folder', { path })
            if (myGen !== scanGeneration) return
            const cached: CachedTrackData[] = await invoke('get_cached_tracks_for_paths', { paths: files })
            if (myGen !== scanGeneration) return 
            audioFiles.value = cached.map(c => ({
                path: c.path,
                fileName: c.file_name,
                title: c.title || undefined,
                artist: c.artist || undefined,
                album: c.album || undefined,
                duration: c.duration,
                track_number: c.track_number || undefined,
            }))
        } catch (e) {
            audioFiles.value = []
        } finally {
            if (myGen === scanGeneration) {
                isScanning.value = false
                scanProgress.value = ''
            }
        }
    }

    async function backgroundScanAll() {
        for (const folder of folders.value) {
            try {
                const files: string[] = await invoke('scan_music_folder', { path: folder.path })
                const cached: CachedTrackData[] = await invoke('get_cached_tracks_for_paths', { paths: files })
                const cachedPaths = new Set(cached.map(c => c.path))
                const uncachedPaths = files.filter(f => !cachedPaths.has(f))

                if (uncachedPaths.length > 0) {
                    const metadataList: RawMetadata[] = await invoke('read_metadata_batch', { Paths: uncachedPaths })
                    const cacheData = metadataList.map(m => ({
                        path: m.path,
                        file_name: m.file_name,
                        title: m.title,
                        artist: m.artist,
                        album: m.album,
                        duration: m.duration,
                        trackNumber: m.track_number,
                    }))
                    await invoke('cache_tracks', { tracks: cacheData })
                    if (selectedFolderPath.value?.startsWith(folder.path)) {
                        await loadFromCache(selectedFolderPath.value)
                    }
                }
            } catch (e) {

            }
        }
    }

    async function scanAndSelect(path: string) {
        selectedFolderPath.value = path
        invoke('set_setting', { key: 'selected-folder', value: path }).catch(() => {})
        const myGen = ++scanGeneration
        isScanning.value = true
        scanProgress.value = '正在扫描文件...'
        try {
            const files: string[] = await invoke('scan_music_folder', { path })
            if (myGen !== scanGeneration) return
            scanProgress.value = `发现 ${files.length} 个音频文件,加载中...`

            const cached: CachedTrackData[] = await invoke('get_cached_tracks_for_paths', { paths: files })
            if (myGen !== scanGeneration) return
            const cachedPaths = new Set(cached.map(c => c.path))

            if (cached.length > 0) {
                audioFiles.value = cached.map(c => ({
                    path: c.path,
                    fileName: c.file_name,
                    title: c.title || undefined,
                    artist: c.artist || undefined,
                    album: c.album || undefined,
                    duration: c.duration,
                    trackNumber: c.track_number || undefined,
                }))
            }

            const uncachedPaths = files.filter(f => !cachedPaths.has(f))

            if (uncachedPaths.length > 0) {
                scanProgress.value = `正在读取 ${uncachedPaths.length} 首歌曲元数据...`

                const unlisten = await listen<RawMetadata[]>('metadata-batch-chunk', (event) => {
                    if (myGen !== scanGeneration) return
                    const chunk = event.payload.map(m => ({
                        path: m.path,
                        fileName: m.file_name,
                        title: m.title || undefined,
                        artist: m.artist || undefined,
                        album: m.album || undefined,
                        duration: m.duration,
                        trackNumber: m.track_number || undefined,
                    }))
                    audioFiles.value = [...audioFiles.value, ...chunk]
                    scanProgress.value = `已加载 ${audioFiles.value.length} / ${files.length} 首...`
                })
                try {
                    const metadataList: RawMetadata[] = await invoke('read_metadata_batch', { paths: uncachedPaths })
                    if (myGen !== scanGeneration) return

                    const allPaths = new Set(audioFiles.value.map(f => f.path))
                    const missing = metadataList.filter(m => !allPaths.has(m.path))
                    if (missing.length > 0) {
                        const missingFiles = missing.map(m => ({
                            path: m.path,
                            fileName: m.file_name,
                            title: m.title || undefined,
                            artist: m.artist || undefined,
                            album: m.album || undefined,
                            duration: m.duration,
                            coverUrl: m.cover || undefined,
                            trackNumber: m.track_number || undefined,
                        }))
                        audioFiles.value = [...audioFiles.value, ...missingFiles]
                    }

                    const cacheData = metadataList.map(m => ({
                        path: m.path,
                        file_name: m.file_name,
                        title: m.title,
                        artist: m.artist,
                        album: m.album,
                        duration: m.duration,
                        track_number: m.track_number,
                    }))
                    invoke('cache_tracks', { tracks: cacheData }).catch(() => {})
                } finally {
                    unlisten()
                }
            }
        } catch (e) {
            console.error('Scan failed:', e)
            audioFiles.value = []
        } finally {
            if (myGen === scanGeneration) {
                isScanning.value = false
                scanProgress.value = ''
            }
        }
    }

    async function globalSearch(query: string) {
        globalSearchQuery.value = query
        if (!query.trim()) {
            globalSearchResults.value = []
            return
        }
        isGlobalSearching.value = true
        try {
            const results: { path: string; file_name: string; title: string | null; artist: string | null; album: string | null; duration: number }[] =
                await invoke('search_tracks', { query: query.trim() })
            globalSearchResults.value = results.map(r => ({
                path: r.path,
                fileName: r.file_name,
                title: r.title || undefined,
                artist: r.artist || undefined,
                album: r.album || undefined,
                duration: r.duration,
            }))
        } catch (e) {
            console.error('Global search failed:', e)
            globalSearchResults.value = []
        } finally {
            isGlobalSearching.value = false
        }
    }

    function clearGlobalSearch() {
        globalSearchQuery.value = ''
        globalSearchResults.value = []
    }

    async function refreshLibrary() {
        await invoke('clear_track_cache')
        if (selectedFolderPath.value) {
            await scanAndSelect(selectedFolderPath.value)
        }
    }

    invoke('cleanup_stale_cache').then(() => {
        backgroundScanAll()
    }).catch(() => {})

    return {
        folders, folderTrees, selectedFolderPath, selectedFolder, audioFiles, isScanning, scanProgress,
        globalSearchQuery, globalSearchResults, isGlobalSearching,
        addFolder, removeFolder, loadFolderTree, selectFolder, globalSearch, clearGlobalSearch, refreshLibrary,
    }
})