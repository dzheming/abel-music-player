<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useLibraryStore } from '../../stores/library'
import { usePlaylistStore } from '../../stores/playlist'
import ContextMenu from '../ContextMenu.vue'
import type { MenuItem } from '../ContextMenu.vue'
import type { LibraryFolderNode } from '../../types'

const props = defineProps<{
    node: LibraryFolderNode
    depth?: number
}>()

const libraryStore = useLibraryStore()
const playlistStore = usePlaylistStore()
const router = useRouter()
const depth = props.depth ?? 0

const showMenu = ref(false)
const menuX = ref(0)
const menuY = ref(0)

function normalizePath(p: string) { return p.replace(/\\/g, '/') }

function shouldAutoExpand(): boolean {
    const selected = libraryStore.selectedFolderPath
    if (!selected) return false
    const ns = normalizePath(selected)
    const np = normalizePath(props.node.path)
    return ns === np || ns.startsWith(np + '/')
}

const expanded = ref(shouldAutoExpand())

function toggle() {
    if (props.node.children.length > 0) {
        expanded.value = !expanded.value
    }
}

function select() {
    libraryStore.selectFolder(props.node.path)
    if (router.currentRoute.value.path !== '/') {
        router.push('/')
    }
}

function onContextMenu(e: MouseEvent) {
    e.preventDefault()
    e.stopPropagation()
    menuX.value = e.clientX
    menuY.value = e.clientY
    showMenu.value = true
}

async function scanNodeFiles(): Promise<string[]> {
    return await invoke('scan_music_folder', { path: props.node.path })
}

const menuItems = computed<MenuItem[]>(() => {
    const items: MenuItem[] = []

    if (playlistStore.playlists.length > 0) {
        items.push({
            label: '添加到播放列表',
            action: () => {},
            children: playlistStore.playlists.map(pl => ({
                label: pl.name,
                action: () => {
                    scanNodeFiles().then(paths => {
                        if (paths.length > 0) playlistStore.addToPlaylist(pl.id, paths)
                    })
                },
            })),
        })
    }

    items.push({
        label: '新建播放列表',
        action: () => {
            const name = `新播放列表 ${Date.now().toString(36)}`
            playlistStore.createPlaylist(name).then(pl => {
                if (pl) {
                    scanNodeFiles().then(paths => {
                        if (paths.length > 0) playlistStore.addToPlaylist(pl.id, paths)
                    })
                }
            })
        },
    })

    const isRootFolder = libraryStore.folders.some(f => f.path === props.node.path)
    if (isRootFolder) {
        items.push({
            label: '从乐库移除',
            action: () => libraryStore.removeFolder(props.node.path),
            danger: true,
        })
    } else {
        items.push({
            label: '排除此目录',
            action: () => libraryStore.excludeFolder(props.node.path),
            danger: true,
        })
    }

    return items
})

const isSelected = computed(() => libraryStore.selectedFolderPath === props.node.path)
const hasChildren = computed(() => props.node.children.length > 0)
</script>

<template>
    <div class="tree-node">
        <div
            class="tree-item"
            :class="{ active: isSelected }"
            :style="{ paddingLeft: (depth * 16 + 8) + 'px'}"
            @click="select"
            @contextmenu="onContextMenu"
        >
            <span
                class="tree-arrow"
                :class="{ expanded, invisible: !hasChildren }"
                @click.stop="toggle"
            >&#9656;</span>
            <span class="tree-icon">&#128193;</span>
            <span class="tree-name">{{ node.name }}</span>
            <span v-if="node.audio_count > 0" class="tree-count">{{ node.audio_count }}</span>
        </div>
        <div v-if="expanded && hasChildren" class="tree-children">
            <FolderTree
                v-for="child in node.children"
                :key="child.path"
                :node="child"
                :depth="depth + 1"
            />
        </div>
        <ContextMenu
            v-if="showMenu"
            :x="menuX"
            :y="menuY"
            :items="menuItems"
            @close="showMenu = false"
        />
    </div>
</template>

<style scoped>
.tree-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background-color var(--transition-fast);
    font-size: 13px;
}

.tree-item:hover {
    background-color: var(--color-bg-hover);
}

.tree-item.active {
    background-color: var(--color-bg-hover);
    font-weight: 500;
}

.tree-arrow {
    font-size: 10px;
    width: 14px;
    text-align: center;
    flex-shrink: 0;
    transition: transform var(--transition-fast);
    color: var(--color-text-tertiary);
}

.tree-arrow.expanded {
    transform: rotate(90deg);
}

.tree-arrow.invisible {
    visibility: hidden;
}

.tree-icon {
    font-size: 14px;
    flex-shrink: 0;
}

.tree-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.tree-count {
    font-size: 11px;
    color: var(--color-text-tertiary);
    flex-shrink: 0;
}
</style>
