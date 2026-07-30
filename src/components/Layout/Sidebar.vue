<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useLibraryStore } from '../../stores/library'
import { useSidebarTab } from '../../composables/useSidebarTab'
import FolderTree from '../Library/FolderTree.vue'
import PlaylistView from '../../views/PlaylistView.vue'
import BrowseView from '../../views/BrowseView.vue'

const libraryStore = useLibraryStore()
const router = useRouter()
const searchInput = ref('')
const { activeTab } = useSidebarTab()

function switchTab(tab: 'library' | 'playlist' | 'browse') {
    activeTab.value = tab
    if (router.currentRoute.value.path !== '/') {
        router.push('/')
    }
}

let searchTimer: ReturnType<typeof setTimeout> | null = null

function onSearchInput() {
    if (searchTimer) clearTimeout(searchTimer)
    searchTimer = setTimeout(() => {
        libraryStore.globalSearch(searchInput.value)
    }, 300)
}

function clearSearch() {
    searchInput.value = ''
    libraryStore.clearGlobalSearch()
}
</script>

<template>
    <aside class="sidebar">
        <div class="sidebar-tabs">
            <button  class="sidebar-tab" :class="{ active: activeTab === 'library' }" @click="switchTab('library')">乐库</button>
            <button  class="sidebar-tab" :class="{ active: activeTab === 'playlist' }" @click="switchTab('playlist')">列表</button>
            <button  class="sidebar-tab" :class="{ active: activeTab === 'browse' }" @click="switchTab('browse')">分类</button>
        </div>

        <div v-show="activeTab === 'library'" class="sidebar-content">
            <div class="folder-trees">
                <div v-for="folder in libraryStore.folders" :key="folder.path" class="library-section">
                    <FolderTree
                        v-if="libraryStore.folderTrees.get(folder.path)"
                        :node="libraryStore.folderTrees.get(folder.path)!"
                    />
                </div>
            </div>

            <div class="sidebar-actions">
                <button class="action-btn" @click="libraryStore.addFolder()">+ 添加文件夹</button>
                <button class="action-btn" @click="router.push('/settings')">&#9881; 设置</button>
            </div>

            <div class="sidebar-search">
                <div class="search-box">
                    <span class="search-icon">&#128269;</span>
                    <input
                        v-model="searchInput"
                        type="text"
                        class="search-input"
                        placeholder="搜索全部乐库"
                        @input="onSearchInput"
                    />
                    <button v-if="searchInput" class="search-clear" @click="clearSearch">&times;</button>
                </div>
            </div>
        </div>

        <div v-show="activeTab === 'playlist'" class="sidebar-content">
            <PlaylistView />
        </div>

        <div v-show="activeTab === 'browse'" class="sidebar-content">
            <BrowseView />
        </div>
    </aside>
</template>

<style scoped>
.sidebar {
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-sidebar);
    border-right: 1px solid var(--color-border);
    height: 100%;
    overflow: hidden;
    transition: background-color var(--transition-normal);
}

.sidebar-tabs {
    display: flex;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
}

.sidebar-tab {
    flex: 1;
    padding: 8px 0;
    font-size: 12px;
    color: var(--color-text-secondary);
    text-align: center;
    border-bottom: 2px solid transparent;
    transition: color var(--transition-fast), border-color var(--transition-fast);
}

.sidebar-tab.active {
    color: var(--color-text-primary);
    border-bottom-color: var(--color-accent);
    font-weight: 500;
}

.sidebar-tab:hover:not(.active) {
    color: var(--color-text-primary);
}

.sidebar-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.folder-trees {
    flex: 1;
    overflow-y: auto;
    padding: 8px 8px 0;
}

.library-section {
    margin-bottom: 8px;
}

.library-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    margin-bottom: 2px;
}

.library-name {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--color-text-tertiary);
    letter-spacing: 0.5px;
}

.library-remove {
    font-size: 14px;
    color: var(--color-text-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    transition: color var(--transition-fast);
}

.library-remove:hover {
    color: var(--color-text-primary);
    background-color: var(--color-bg-hover);
}

.sidebar-actions {
    padding: 4px 8px;
}

.action-btn {
    display: block;
    width: 100%;
    text-align: left;
    padding: 6px 12px;
    border-radius: var(--radius-md);
    font-size: 13px;
    color: var(--color-text-secondary);
    transition: background-color var(--transition-fast), color var(--transition-fast);
}

.action-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
}

.sidebar-search {
    padding: 8px 12px 12px;
    border-top: 1px solid var(--color-border);
}

.search-box {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-radius: var(--radius-md);
    background-color: var(--color-bg-hover);
    transition: background-color var(--transition-fast);
}

.search-box:focus-within {
    outline: 1px solid var(--color-accent);
}

.search-icon {
    font-size: 12px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
}

.search-input {
    flex: 1;
    border: none;
    background: none;
    outline: none;
    font-size: 12px;
    color: var(--color-text-primary);
    font-family: inherit;
}

.search-input::placeholder {
    color: var(--color-text-tertiary);
}

.search-clear {
    font-size: 14px;
    color: var(--color-text-tertiary);
    padding: 0 2px;
    flex-shrink: 0;
}

.search-clear:hover {
    color: var(--color-text-primary);
}
</style>