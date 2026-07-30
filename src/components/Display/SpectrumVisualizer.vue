<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { usePlayerStore } from '../../stores/player'

const playerStore = usePlayerStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
let animationId: number | null = null
let analyser: AnalyserNode | null = null
let accentColor = '#0a84ff'

let accentObserver: MutationObserver | null = null

function refreshAccentColor() {
    accentColor = getComputedStyle(document.documentElement).getPropertyValue('--color-accent').trim() || '#0a84ff'
}

onMounted(() => {
    try {
        analyser = playerStore.getAnalyser()
    } catch (e) {
        console.error('Failed to init analyser: ', e)
    }
    refreshAccentColor()
    accentObserver = new MutationObserver(() => refreshAccentColor())
    accentObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['style'] })

    if (canvasRef.value) {
        intersectionObserver = new IntersectionObserver((entries) => {
            const visible = entries[0]?.isIntersecting ?? false
            if (visible && animationId === null) {
                animationId = requestAnimationFrame(draw)
            } else if (!visible && animationId !== null) {
                cancelAnimationFrame(animationId)
                animationId = null
            }
        })
        intersectionObserver.observe(canvasRef.value)
    }
})

let intersectionObserver: IntersectionObserver | null = null

onUnmounted(() => {
    if (animationId) {
        cancelAnimationFrame(animationId)
        animationId = null
    }
    if (accentObserver) {
        accentObserver.disconnect()
        accentObserver = null
    }
    if (intersectionObserver) {
        intersectionObserver.disconnect()
        intersectionObserver = null
    }
})

function draw() {
    animationId = requestAnimationFrame(draw)

    if (!canvasRef.value || !analyser) return

    const canvas = canvasRef.value
    const width = canvas.clientWidth
    const height = canvas.clientHeight
    if (width === 0 || height === 0) return

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const scaledW = width * window.devicePixelRatio
    const scaledH = height * window.devicePixelRatio
    if (canvas.width !== scaledW || canvas.height !== scaledH) {
        canvas.width = scaledW
        canvas.height = scaledH
    }
    ctx.setTransform(window.devicePixelRatio, 0, 0, window.devicePixelRatio, 0, 0)
    ctx.clearRect(0, 0, width, height)

    const bufferLength = analyser.frequencyBinCount
    const dataArray = new Uint8Array(bufferLength)
    analyser.getByteFrequencyData(dataArray)

    const startX = width * 0.04
    const availableWidth = width * 0.96
    const barCount = Math.min(Math.floor(availableWidth / 6), bufferLength)
    const totalUnit = availableWidth / barCount
    const barWidth = totalUnit * 0.65
    const gap = totalUnit * 0.35

    for (let i = 0; i < barCount; i++) {
        const barHeight = (dataArray[i] / 255) * height * 0.85
        const x = startX + i * (barWidth + gap)
        if (x > availableWidth) break
        if (barHeight < 1) continue
        const y = height - barHeight

        ctx.fillStyle = accentColor
        ctx.globalAlpha = 0.5 + (dataArray[i] / 255) * 0.5
        ctx.beginPath()
        ctx.roundRect(x, y, barWidth, barHeight, 2)
        ctx.fill()
    }

    ctx.globalAlpha = 1
}
</script>

<template>
    <canvas ref="canvasRef" class="spectrum-canvas"></canvas>
</template>

<style scoped>
.spectrum-canvas {
    width: 100%;
    height: 100%;
    display: block;
}
</style>
