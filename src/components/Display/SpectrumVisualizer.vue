<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { usePlayerStore } from '../../stores/player'

const playerStore = usePlayerStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
let animationId: number | null = null
let analyser: AnalyserNode | null = null
let accentColor = '#0a84ff'
let peaks: Float32Array = new Float32Array(0)
let lastTime = 0

let accentObserver: MutationObserver | null = null
let intersectionObserver: IntersectionObserver | null = null

function refreshAccentColor() {
    accentColor = getComputedStyle(document.documentElement).getPropertyValue('--color-accent').trim() || '#0a84ff'
}

function ensureAnalyser() {
    if (analyser) return true
    try {
        analyser = playerStore.getAnalyser()
        return true
    } catch {
        return false
    }
}

onMounted(() => {
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

    if (!canvasRef.value) return

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

    // 仅在播放时渲染频谱
    if (!playerStore.isPlaying) {
        peaks = new Float32Array(0)
        lastTime = 0
        return
    }

    if (!ensureAnalyser()) return

    const a = analyser
    if (!a) return

    const bufferLength = a.frequencyBinCount
    const dataArray = new Uint8Array(bufferLength)
    a.getByteFrequencyData(dataArray)

    const startX = width * 0.04
    const availableWidth = width * 0.96
    const barCount = Math.min(Math.floor(availableWidth / 6), bufferLength)
    const totalUnit = availableWidth / barCount
    const barWidth = totalUnit * 0.65
    const gap = totalUnit * 0.35

    const now = performance.now()
    const dt = lastTime > 0 ? Math.min((now - lastTime) / 1000, 0.05) : 0
    lastTime = now

    if (peaks.length !== barCount) {
        peaks = new Float32Array(barCount)
    }

    // 峰值下落速率:每秒下落画布高度的 25%
    const fallSpeed = height * 0.25
    const peakCapHeight = 2

    // 先更新所有峰值并绘制柱状
    ctx.fillStyle = accentColor
    for (let i = 0; i < barCount; i++) {
        const barHeight = (dataArray[i] / 255) * height * 0.85
        const x = startX + i * (barWidth + gap)
        if (x > availableWidth) break

        if (barHeight >= peaks[i]) {
            peaks[i] = barHeight
        } else {
            peaks[i] = Math.max(barHeight, peaks[i] - fallSpeed * dt)
        }

        if (barHeight < 1) continue
        const y = height - barHeight

        ctx.globalAlpha = 0.5 + (dataArray[i] / 255) * 0.5
        ctx.fillRect(x, y, barWidth, barHeight)
    }

    // 绘制峰值帽(短横线)
    ctx.globalAlpha = 1
    for (let i = 0; i < barCount; i++) {
        const peakHeight = peaks[i]
        if (peakHeight < 2) continue
        const x = startX + i * (barWidth + gap)
        if (x > availableWidth) break
        const y = height - peakHeight
        ctx.fillRect(x, y, barWidth, peakCapHeight)
    }
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
