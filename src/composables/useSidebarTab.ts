import { useUiStore } from '../stores/ui'

export type { SidebarTab } from '../stores/ui'

export function useSidebarTab() {
    const uiStore = useUiStore()
    return {
        activeTab: uiStore.activeTab,
        setActiveTab: uiStore.setActiveTab
    }
}
