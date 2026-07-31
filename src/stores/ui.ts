import { defineStore } from 'pinia'
import { ref } from 'vue'

export type SidebarTab = 'library' | 'playlist' | 'browse'

export const useUiStore = defineStore('ui', () => {
    const activeTab = ref<SidebarTab>('library')

    function setActiveTab(tab: SidebarTab) {
        activeTab.value = tab
    }

    return { activeTab, setActiveTab }
})
