<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { usePlayerStore } from '../../stores/player'

const playerStore = usePlayerStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
let animationId: number | null = null
let analyser: AnalyserNode | null = null
let accentColor = '#0a84ff'
let frameCount = 0

function refreshAccentColor() {
    accentColor = getComputedStyle(document.documentElement).getPropertyValue('--color-accent').trim() || '#0a84ff'
    frameCount = 0
}

onMounted(() => {
    try {
        analyser = playerStore.getAnalyser()
    } catch (e) {
        console.error('Failed to init analyser: ', e)
    }
    refreshAccentColor()
    animationId = requestAnimationFrame(draw)
})

onUnmounted(() => {
    if (animationId) cancelAnimationFrame(animationId)
})

function draw() {
    animationId = requestAnimationFrame(draw)

    if (++frameCount % 30 === 0) refreshAccentColor()

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