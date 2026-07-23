import { ref, computed } from 'vue'
import { usePlaylistStore } from '../stores/playlist'
import type { MenuItem } from '../components/ContextMenu.vue'

export function useTrackContextMenu(getPath: () => string) {
    const playlistStore = usePlaylistStore()

    const showMenu = ref(false)
    const menuX = ref(0)
    const menuY = ref(0)

    function onContextMenu(e: MouseEvent) {
        e.preventDefault()
        menuX.value = e.clientX
        menuY.value = e.clientY
        showMenu.value = true
    }

    const menuItems = computed<MenuItem[]>(() => {
        const path = getPath()
        const items: MenuItem[] = [
            { label: '添加到默认列表', action: () => playlistStore.addToDefault([path]) },
        ]
        if (playlistStore.playlists.length > 0) {
            items.push({
                label: '添加到播放列表',
                action: () => {},
                children: playlistStore.playlists.map(pl => ({
                    label: pl.name,
                    action: () => playlistStore.addToPlaylist(pl.id, [path]),
                })),
            })
        }
        items.push({
            label: '新建播放列表',
            action: async () => {
                const name = `新播放列表 ${playlistStore.playlists.length + 1}`
                const pl = await playlistStore.createPlaylist(name)
                if (pl) await playlistStore.addToPlaylist(pl.id, [path])
            },
        })
        return items
    })

    return { showMenu, menuX, menuY, onContextMenu, menuItems }
}
