<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
    title: string
    album: string
    artist: string
    audioPath: string
}>()

const emit = defineEmits<{
    (e: 'select', lrc: string): void
    (e: 'close'): void
}>()

interface SearchResult {
    id: number
    name: string
    artist: string
    album: string
}

const searchTitle = ref(props.title || '')
const searchAlbum = ref(props.album || '')
const searchArtist = ref(props.artist || '')
const results = ref<SearchResult[]>([])
const isSearching = ref(false)
const isFetching = ref<number | null>(null)
const error = ref('')

async function doSearch() {
    const q = [searchTitle.value, searchAlbum.value, searchArtist.value].filter(Boolean).join(' ')
    if (!q.trim()) return

    isSearching.value = true
    error.value = ''
    results.value = []

    try {
        results.value = await invoke('search_netease_lyrics', { query: q, artistFilter: searchArtist.value.trim() })
    } catch (e: any) {
        error.value = '搜索失败: ' + (e?.toString() || '未知错误')
    } finally {
        isSearching.value = false
    }
}

async function selectResult(item: SearchResult) {
    isFetching.value = item.id
    error.value = ''
    try {
        const lrc: string | null = await invoke('fetch_netease_lyric', {
            songId: item.id,
            audioPath: props.audioPath,
        })
        if (lrc) {
            emit('select', lrc)
        } else {
            error.value = '该歌曲暂无歌词'
        }
    } catch (e: any) {
        error.value = '获取歌词失败: ' + (e?.toString() || '未知错误')
    } finally {
        isFetching.value = null
    }
}

function onOverlayClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('lyrics-search-overlay')) {
        emit('close')
    }
}
</script>

<template>
    <Teleport to="body">
        <div class="lyrics-search-overlay" @click="onOverlayClick">
            <div class="lyrics-search-dialog">
                <div class="dialog-header">
                    <span class="dialog-title">搜索歌词</span>
                    <button class="dialog-close" @click="emit('close')">&times;</button>
                </div>

                <div class="dialog-search">
                    <div class="search-row">
                        <label>歌名</label>
                        <input v-model="searchTitle" @keyup.enter="doSearch" placeholder="歌曲名称" />
                    </div>
                    <div class="search-row">
                        <label>专辑</label>
                        <input v-model="searchAlbum" @keyup.enter="doSearch" placeholder="专辑名称" />
                    </div>
                    <div class="search-row">
                        <label>歌手</label>
                        <input v-model="searchArtist" @keyup.enter="doSearch" placeholder="歌手名称" />
                    </div>
                    <button class="search-btn" :disabled="isSearching" @click="doSearch">
                        {{ isSearching ? '搜索中...' : '搜索' }}
                    </button>
                </div>

                <div v-if="error" class="dialog-error">{{ error }}</div>

                <div class="dialog-results">
                    <div v-if="results.length === 0 && !isSearching" class="results-empty">
                        输入关键词搜索歌词
                    </div>
                    <div
                        v-for="item in results"
                        :key="item.id"
                        class="result-item"
                    >
                        <div class="result-info">
                            <span class="result-name">{{ item.name }}</span>
                            <span class="result-meta">{{ item.artist }} {{ item.album ? ' - ' + item.album : '' }}</span>
                        </div>
                        <button
                            class="result-use-btn"
                            :disabled="isFetching === item.id"
                            @click="selectResult(item)"
                        >
                            {{ isFetching === item.id ? '获取中...' : '使用' }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<style scoped>
.lyrics-search-overlay {
    position: fixed;
    inset: 0;
    z-index: 999999;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
}

.lyrics-search-dialog {
    width: 480px;
    max-height: 70vh;
    background: var(--color-bg-primary);
    border-radius: var(--radius-lg, 12px);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
}

.dialog-title {
    font-size: 15px;
    font-weight: 600;
}

.dialog-close {
    font-size: 20px;
    color: var(--color-text-secondary);
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm, 4px);
    transition: background-color 0.15s;
}

.dialog-close:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
}

.dialog-search {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
}

.search-row {
    display: flex;
    align-items: center;
    gap: 10px;
}

.search-row label {
    font-size: 13px;
    color: var(--color-text-secondary);
    width: 36px;
    flex-shrink: 0;
}

.search-row input {
    flex: 1;
    padding: 6px 10px;
    font-size: 13px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm, 4px);
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
    outline: none;
    font-family: inherit;
    transition: border-color 0.15s;
}

.search-row input:focus {
    border-color: var(--color-accent);
}

.search-btn {
    align-self: flex-end;
    padding: 6px 16px;
    font-size: 13px;
    background: var(--color-accent);
    color: #fff;
    border-radius: var(--radius-sm, 4px);
    font-weight: 500;
    transition: background-color 0.15s;
}

.search-btn:hover:not(:disabled) {
    background: var(--color-accent-hover);
}

.search-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.dialog-error {
    padding: 8px 20px;
    font-size: 12px;
    color: #e53935;
    flex-shrink: 0;
}

.dialog-results {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
}

.results-empty {
    padding: 32px 20px;
    text-align: center;
    font-size: 13px;
    color: var(--color-text-tertiary);
}

.result-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 20px;
    gap: 12px;
    transition: background-color 0.15s;
}

.result-item:hover {
    background: var(--color-bg-hover);
}

.result-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.result-name {
    font-size: 13px;
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.result-meta {
    font-size: 12px;
    color: var(--color-text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.result-use-btn {
    flex-shrink: 0;
    padding: 4px 12px;
    font-size: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm, 4px);
    color: var(--color-text-secondary);
    transition: all 0.15s;
}

.result-use-btn:hover:not(:disabled) {
    border-color: var(--color-accent);
    color: var(--color-accent);
}

.result-use-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}
</style>