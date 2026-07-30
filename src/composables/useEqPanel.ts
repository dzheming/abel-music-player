import { ref, onMounted, onUnmounted } from 'vue'

export function useEqPanel() {
    const showEq = ref(false)
    const eqBtnRef = ref<HTMLElement | null>(null)
    const eqPanelRef = ref<HTMLElement | null>(null)

    function toggleEqPanel() {
        showEq.value = !showEq.value
    }

    function onClickOutside(e: MouseEvent) {
        if (
            showEq.value &&
            eqPanelRef.value && !eqPanelRef.value.contains(e.target as Node) &&
            eqBtnRef.value && !eqBtnRef.value.contains(e.target as Node)
        ) {
            showEq.value = false
        }
    }

    onMounted(() => document.addEventListener('mousedown', onClickOutside))
    onUnmounted(() => document.removeEventListener('mousedown', onClickOutside))

    return { showEq, eqBtnRef, eqPanelRef, toggleEqPanel }
}
