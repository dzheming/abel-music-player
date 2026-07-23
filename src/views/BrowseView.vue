<script setup lang="ts">
import { onMounted } from 'vue'
import { useBrowseStore } from '../stores/browse'

const browseStore = useBrowseStore()

async function autoSelect() {
    if (browseStore.viewMode === 'artists' && !browseStore.currentArtist && browseStore.artists.length > 0) {
        browseStore.selectArtist(browseStore.artists[0].artist)
    } else if (browseStore.viewMode === 'albums' && !browseStore.currentAlbum && browseStore.albums.length > 0) {
        browseStore.selectAlbum(browseStore.albums[0].album)
    }
}

onMounted(async () => {
    await Promise.all([browseStore.loadArtists(), browseStore.loadAlbums()])
    autoSelect()
})

function switchView(mode: 'artists' | 'albums') {
    browseStore.viewMode = mode
    autoSelect()
}
</script>

<template>
    <div class="browse-view">
        <div class="browse-tabs">
            <button
                class="browse-tab"
                :class="{ active: browseStore.viewMode === 'artists' }"
                @click="switchView('artists')"
            >歌手</button>
            <button
                class="browse-tab"
                :class="{ active: browseStore.viewMode === 'albums' }"
                @click="switchView('albums')"
            >专辑</button>    
        </div>

        <div class="browse-list">
            <template v-if="browseStore.viewMode === 'artists'">
                <div
                    v-for="a in browseStore.artists"
                    :key="a.artist"
                    class="browse-item"
                    :class="{ active: browseStore.currentArtist === a.artist }"
                    @click="browseStore.selectArtist(a.artist)"
                >
                    <span class="browse-name">{{ a.artist }}</span>
                    <span class="browse-count">{{ a.track_count }}</span>
                </div>
                <div v-if="browseStore.artists.length === 0" class="empty-hint">
                    暂无数据
                </div>
            </template>

            <template v-else>
                <div
                    v-for="a in browseStore.albums"
                    :key="a.album"
                    class="browse-item"
                    :class="{ active: browseStore.currentAlbum === a.album }"
                    @click="browseStore.selectAlbum(a.album)"
                >
                    <span class="browse-name">{{ a.album }}</span>
                    <span class="browse-count">{{ a.track_count }}</span>
                </div>
                <div v-if="browseStore.albums.length === 0" class="empty-hint">
                    暂无数据
                </div>
            </template>
        </div>
    </div>
</template>

<style scoped>
.browse-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.browse-tabs {
    display: flex;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
}

.browse-tab {
    flex: 1;
    padding: 6px 0;
    font-size: 12px;
    color: var(--color-text-secondary);
    text-align: center;
    border-bottom: 2px solid transparent;
    transition: color var(--transition-fast), border-color var(--transition-fast);
}

.browse-tab.active {
    color: var(--color-text-primary);
    border-bottom-color: var(--color-accent);
    font-weight: 500;
}

.browse-tab:hover:not(.active) {
    color: var(--color-text-primary);
}

.browse-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px;
}

.browse-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: 13px;
    transition: background-color var(--transition-fast);
}

.browse-item:hover {
    background-color: var(--color-bg-hover);
}

.browse-item.active {
    background-color: var(--color-bg-hover);
    font-weight: 500;
}

.browse-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
}

.browse-count {
    font-size: 11px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
    margin-left: 8px;
}

.empty-hint {
    padding: 16px;
    text-align: center;
    font-size: 12px;
    color: var(--color-text-tertiary);
}
</style>