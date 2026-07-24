<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps<{
    items: any[]
    itemHeight: number
    overscan?: number
}>()

const emit = defineEmits<{
    (e: 'visible-range', start: number, end: number): void
}>()

const container = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const containerHeight = ref(0)
const overscan = computed(() => props.overscan ?? 10)

const totalHeight = computed(() => props.items.length * props.itemHeight)

const startIndex = computed(() => 
    Math.max(0, Math.floor(scrollTop.value / props.itemHeight) - overscan.value)
)

const endIndex = computed(() => 
    Math.min(
        props.items.length,
        Math.ceil((scrollTop.value + containerHeight.value) / props.itemHeight) + overscan.value
    )
)

const offsetY = computed(() => startIndex.value * props.itemHeight)

const visibleItems = computed(() => 
    props.items.slice(startIndex.value, endIndex.value).map((item, i) => ({
        item,
        index: startIndex.value + i
    }))
)

function onScroll() {
    if (container.value) {
        scrollTop.value = container.value.scrollTop
    }
}

function updateHeight() {
    if (container.value) {
        containerHeight.value = container.value.clientHeight
    }
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
    updateHeight()
    resizeObserver = new ResizeObserver(updateHeight)
    if (container.value) resizeObserver.observe(container.value)
})

onUnmounted(() => {
    resizeObserver?.disconnect()
})

watch([startIndex, endIndex], ([s, e]) => {
    emit('visible-range', s, e)
})

function scrollToIndex(index: number, behavior: ScrollBehavior = 'auto') {
    if (container.value) {
        const top = index * props.itemHeight
        container.value.scrollTo({ top, behavior })
    }
}

defineExpose({ scrollToIndex, container })
</script>


<template>
    <div ref="container" class="virtual-list-container" @scroll="onScroll">
        <div class="virtual-list-spacer" :style="{ height: totalHeight + 'px'}">
            <div class="virtual-list-content" :style="{ transform: `translateY(${offsetY}px)` }">
                <template v-for="{ item, index } in visibleItems" :key="index">
                    <slot :item="item" :index="index" />
                </template>
            </div>
        </div>
    </div>
</template>

<style scoped>
.virtual-list-container {
    overflow-y: auto;
    height: 100%;
}

.virtual-list-spacer {
    position: relative;
}

.virtual-list-content {
    position: absolute;
    left: 0;
    right: 0;
    top: 0;
}
</style>