import { ref } from 'vue'

export type SidebarTab = 'library' | 'playlist' | 'browse'

const activeTab = ref<SidebarTab>('library')

export function useSidebarTab() {
    return { activeTab }
}
