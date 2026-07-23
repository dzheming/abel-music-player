<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'

export interface MenuItem {
    label: string
    action: () => void
    danger?: boolean
    children?: MenuItem[]
}

const props = defineProps<{
    x: number
    y: number
    items: MenuItem[]
}>()

const emit = defineEmits<{ close: [] }>()
const menuRef = ref<HTMLElement | null>(null)
const expandedIndex = ref<number | null>(null)
const submenuLeft = ref(false)

function handleClick(item: MenuItem) {
    if (item.children) {
        return
    }
    item.action()
    emit('close')
}

function handleMouseEnter(index: number, item: MenuItem) {
    if (item.children) {
        expandedIndex.value = index
        nextTick(() => {
            const el = menuRef.value?.querySelector('.context-menu-item.has-children') as HTMLElement | null
            if (el) {
                submenuLeft.value = (el.getBoundingClientRect().right + 140) > window.innerWidth
            }
        })
    } else {
        expandedIndex.value = null
    }
}

function onClickOutside(e: MouseEvent) {
    if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
        emit('close')
    }
}

onMounted(() => {
    nextTick(() => document.addEventListener('mousedown', onClickOutside))
})

onUnmounted(() => {
    document.removeEventListener('mousedown', onClickOutside)
})
</script>

<template>
    <Teleport to="body">
        <div ref="menuRef" class="context-menu" :style="{ left: props.x + 'px', top: props.y + 'px' }">
            <div
                v-for="(item, index) in props.items"
                :key="index"
                class="context-menu-item"
                :class="{ danger: item.danger, 'has-children' : !!item.children }"
                @click="handleClick(item)"
                @mouseenter="handleMouseEnter(index, item)"
            >
                <span>{{ item.label }}</span>
                <span v-if="item.children" class="arrow">&#9656;</span>
                <div
                    v-if="item.children && expandedIndex === index"
                    class="context-submenu"
                    :class="{ left: submenuLeft }"
                >
                    <div
                        v-for="(child, ci) in item.children"
                        :key="ci"
                        class="context-menu-item"
                        :class="{ danger: child.danger }"
                        @click.stop="() => { child.action(); emit('close') }"
                    >
                        <span>{{ child.label }}</span>
                    </div>
                </div>
            </div>
        </div>
    </Teleport>
</template>

<style>
.context-menu {
    position: fixed;
    z-index: 99999;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0,0,0,0.2);
    padding: 4px;
    min-width: 160px;
}

.context-menu-item {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 12px;
    font-size: 13px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--color-text-primary);
    user-select: none;
}

.context-menu-item:hover {
    background-color: var(--color-bg-hover);
}

.context-menu-item.danger {
    color: #e53935;
}

.context-menu-item.arrow {
    font-size: 10px;
    color: var(--color-text-tertiary);
}

.context-submenu {
    position: absolute;
    left: 100%;
    top: 0;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 4px 16px rgba(0,0,0,0.2);
    padding: 4px;
    min-width: 140px;
}

.context-submenu.left {
    left: auto;
    right: 100%;
}
</style>