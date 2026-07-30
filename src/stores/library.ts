import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { useBrowseStore } from './browse'
import { usePlaylistStore } from './playlist'
import { usePlayerStore } from './player'
import { toTrack } from '../types'
import type { Track, RawTrack, LibraryFolder, LibraryFolderNode } from '../types'

export const useLibraryStore = defineStore('library', () => {
    const folders = ref<LibraryFolder[]>([])
    const folderTrees = ref<Map<string, LibraryFolderNode>>(new Map())
    const selectedFolderPath = ref<string | null>(null)
    const audioFiles = ref<Track[]>([])
    const isScanning = ref(false)
    const scanProgress = ref('')
    const globalSearchQuery = ref('')
    const globalSearchResults = ref<Track[]>([])
    const isGlobalSearching = ref(false)
    let scanGeneration = 0

    const selectedFolder = computed(() =>
        folders.value.find(f => f.path === selectedFolderPath.value) || null
    )

    async function initLibrary() {
        try {
            folders.value = await invoke('get_library_folders')
            const selectedRaw = await invoke('get_setting', { key: 'selected-folder' })
            if (selectedRaw) {
                selectedFolderPath.value = selectedRaw as string
            }
            for (const folder of folders.value) {
                await syncAndLoadTree(folder.path)
            }
            if (selectedRaw) {
                await loadFromCache(selectedRaw as string)
            }
        } catch (e) {
            console.error('Failed to init library:', e)
        }

        try {
            await invoke('cleanup_stale_cache')
            await backgroundScanAll()
        } catch (e) {
            console.error('Background scan failed:', e)
        }
    }

    async function syncAndLoadTree(rootPath: string) {
        try {
            await invoke('sync_library_folder', { rootPath })
            const tree: LibraryFolderNode = await invoke('get_folder_tree', { rootPath })
            folderTrees.value.set(rootPath, tree)
        } catch (e) {
            console.error('Failed to sync/load folder tree:', e)
        }
    }

    async function addFolder() {
        const selected = await open({ directory: true, multiple: false })
        if (!selected) return

        const path = selected as string
        const rootPath: string = await invoke('add_library_folder', { path })

        folders.value = await invoke('get_library_folders')
        await syncAndLoadTree(rootPath)
        await scanAndSelect(path)
        useBrowseStore().refresh()
    }

    async function removeFolder(path: string) {
        await invoke('remove_library_folder', { path })
        const playerStore = usePlayerStore()
        if (playerStore.currentTrack?.path.startsWith(path)) {
            playerStore.stop()
            playerStore.setPlaylist([], 0)
        }
        const playlistStore = usePlaylistStore()
        await playlistStore.loadPlaylists()
        if (playlistStore.currentPlaylistId) {
            await playlistStore.selectPlaylist(playlistStore.currentPlaylistId)
        }

        folders.value = await invoke('get_library_folders')
        folderTrees.value.delete(path)
        if (selectedFolderPath.value?.startsWith(path)) {
            if (folders.value.length > 0) {
                await selectFolder(folders.value[0].path)
            } else {
                selectedFolderPath.value = null
                invoke('set_setting', { key: 'selected-folder', value: '' }).catch(() => {})
                audioFiles.value = []
            }
        }
        useBrowseStore().refresh()
    }

    async function excludeFolder(path: string) {
        await invoke('exclude_folder', { path })
        const playerStore = usePlayerStore()
        if (playerStore.currentTrack?.path.startsWith(path)) {
            playerStore.stop()
            playerStore.setPlaylist([], 0)
        }
        const playlistStore = usePlaylistStore()
        await playlistStore.loadPlaylists()
        if (playlistStore.currentPlaylistId) {
            await playlistStore.selectPlaylist(playlistStore.currentPlaylistId)
        }

        // Reload folder trees
        const rootFolder = folders.value.find(f => path.startsWith(f.path))
        if (rootFolder) {
            const tree: LibraryFolderNode = await invoke('get_folder_tree', { rootPath: rootFolder.path })
            folderTrees.value.set(rootFolder.path, tree)
        }

        if (selectedFolderPath.value?.startsWith(path)) {
            if (rootFolder) {
                await selectFolder(rootFolder.path)
            } else if (folders.value.length > 0) {
                await selectFolder(folders.value[0].path)
            } else {
                selectedFolderPath.value = null
                invoke('set_setting', { key: 'selected-folder', value: '' }).catch(() => {})
                audioFiles.value = []
            }
        }
        useBrowseStore().refresh()
    }

    async function restoreFolder(path: string) {
        await invoke('restore_folder', { path })
        const rootFolder = folders.value.find(f => path.startsWith(f.path))
        if (rootFolder) {
            await syncAndLoadTree(rootFolder.path)
        }
        if (selectedFolderPath.value) {
            await loadFromCache(selectedFolderPath.value)
        }
        useBrowseStore().refresh()
    }

    async function selectFolder(path: string) {
        selectedFolderPath.value = path
        globalSearchQuery.value = ''
        globalSearchResults.value = []
        invoke('set_setting', { key: 'selected-folder', value: path }).catch(() => {})
        await loadFromCache(path)
    }

    async function loadFromCache(path: string) {
        const myGen = ++scanGeneration
        isScanning.value = true
        scanProgress.value = '加载中...'
        try {
            const files: string[] = await invoke('scan_music_folder', { path })
            if (myGen !== scanGeneration) return
            const cached: RawTrack[] = await invoke('get_cached_tracks_for_paths', { paths: files })
            if (myGen !== scanGeneration) return
            audioFiles.value = cached.map(toTrack)
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
                const cached: RawTrack[] = await invoke('get_cached_tracks_for_paths', { paths: files })
                const cachedPaths = new Set(cached.map(c => c.path))
                const uncachedPaths = files.filter(f => !cachedPaths.has(f))

                if (uncachedPaths.length > 0) {
                    const metadataList: RawTrack[] = await invoke('read_metadata_batch', { paths: uncachedPaths })
                    await invoke('cache_tracks', { tracks: metadataList })
                    if (selectedFolderPath.value?.startsWith(folder.path)) {
                        await loadFromCache(selectedFolderPath.value)
                    }
                }
            } catch (e) {
                console.error('Background scan failed for folder:', folder.path, e)
            }
        }
    }

    async function scanAndSelect(path: string) {
        selectedFolderPath.value = path
        invoke('set_setting', { key: 'selected-folder', value: path }).catch(() => {})
        const myGen = ++scanGeneration
        isScanning.value = true
        scanProgress.value = '正在扫描文件...'
        audioFiles.value = []

        try {
            const files: string[] = await invoke('scan_music_folder', { path })
            if (myGen !== scanGeneration) return
            scanProgress.value = `发现 ${files.length} 个文件,加载中...`

            const uncachedPaths: string[] = []
            const BATCH = 200
            for (let i = 0; i < files.length; i += BATCH) {
                if (myGen !== scanGeneration) return
                const batch = files.slice(i, i + BATCH)
                const cached: RawTrack[] = await invoke('get_cached_tracks_for_paths', { paths: batch })
                if (myGen !== scanGeneration) return
                const cachedPaths = new Set(cached.map(c => c.path))

                if (cached.length > 0) {
                    audioFiles.value.push(...cached.map(toTrack))
                }

                for (const p of batch) {
                    if (!cachedPaths.has(p)) uncachedPaths.push(p)
                }
                scanProgress.value = `已加载 ${audioFiles.value.length} / ${files.length} 首...`
            }

            if (uncachedPaths.length > 0) {
                scanProgress.value = `正在读取 ${uncachedPaths.length} 首歌曲元数据...`

                const unlistenMeta = await listen<RawTrack[]>('metadata-batch-chunk', (event) => {
                    if (myGen !== scanGeneration) return
                    audioFiles.value = [...audioFiles.value, ...event.payload.map(toTrack)]
                    scanProgress.value = `已加载 ${audioFiles.value.length} / ${files.length} 首...`
                })
                try {
                    const metadataList: RawTrack[] = await invoke('read_metadata_batch', { paths: uncachedPaths })
                    if (myGen !== scanGeneration) return

                    const allPaths = new Set(audioFiles.value.map(f => f.path))
                    const missing = metadataList.filter(m => !allPaths.has(m.path))
                    if (missing.length > 0) {
                        audioFiles.value = [...audioFiles.value, ...missing.map(toTrack)]
                    }

                    await invoke('cache_tracks', { tracks: metadataList })
                } finally {
                    unlistenMeta()
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
            const results: RawTrack[] = await invoke('search_tracks', { query: query.trim() })
            globalSearchResults.value = results.map(toTrack)
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
        for (const folder of folders.value) {
            await syncAndLoadTree(folder.path)
        }
        await invoke('clear_track_cache')
        if (selectedFolderPath.value) {
            await scanAndSelect(selectedFolderPath.value)
        }
        await backgroundScanAll()
    }

    return {
        folders, folderTrees, selectedFolderPath, selectedFolder, audioFiles, isScanning, scanProgress,
        globalSearchQuery, globalSearchResults, isGlobalSearching,
        addFolder, removeFolder, excludeFolder, restoreFolder, selectFolder,
        globalSearch, clearGlobalSearch, refreshLibrary, initLibrary,
    }
})
