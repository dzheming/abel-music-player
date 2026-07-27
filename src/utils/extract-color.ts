function ensureMinLightness(rgb: [number, number, number], minL: number): [number, number, number] {
    const [r, g, b] = rgb
    const rf = r / 255, gf = g / 255, bf = b / 255
    const max = Math.max(rf, gf, bf), min = Math.min(rf, gf, bf)
    let l = (max + min) / 2

    if (l >= minL) return rgb

    let h = 0, s = 0
    if (max !== min) {
        const d = max - min
        s =  l > 0.5 ? d / (2 - max - min) : d / (max + min)
        if (max === rf) h = ((gf - bf) / d + (gf < bf ? 6 : 0)) / 6
        else if (max === gf) h = ((bf - rf) / d + 2) / 6
        else h = ((rf -gf) / d + 4) / 6
    }

    l = minL
    const hue2rgb = (p: number, q: number, t: number) => {
        if (t < 0) t += 1
        if (t > 1) t -= 1
        if (t < 1 / 6) return p + (q - p) * 6 * t
        if (t < 1 / 2) return q
        if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
        return p
    }
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s
    const p = 2 * l - q
    return [
        Math.round(hue2rgb(p, q, h + 1 / 3) * 255),
        Math.round(hue2rgb(p, q, h) * 255),
        Math.round(hue2rgb(p, q, h - 1 / 3) * 255),
    ]
}

function complementary(r: number, g: number, b: number): [number, number, number] {
    let h: number, s: number, l: number
    const rf = r / 255, gf = g / 255, bf = b / 255
    const max = Math.max(rf, gf, bf), min = Math.min(rf, gf, bf)
    l = (max + min) / 2

    if (max === min) {
        h = 0; s = 0
    } else {
        const d = max - min
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
        if (max === rf) h = ((gf -bf) / d + (gf < bf ? 6 : 0)) / 6
        else if (max === gf) h = ((bf -rf) / d + 2) / 6
        else h = ((rf - gf) / d + 4) / 6
    }

    h = (h + 0.5) % 1.0

    const hue2rgb = (p: number, q: number, t: number) => {
        if (t < 0) t += 1
        if (t > 1) t -= 1
        if (t < 1 / 6) return p + (q - p) * 6 * t
        if (t < 1 / 2) return q
        if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6
        return p
    }

    let ro: number, go: number, bo: number
    if (s === 0) {
        ro = go = bo = l
    } else {
        const q = l < 0.5 ? l * (1 + s) : l + s -l * s
        const p = 2 * l - q
        ro = hue2rgb(p, q, h + 1 / 3)
        go = hue2rgb(p, q, h)
        bo = hue2rgb(p, q, h - 1 / 3)
    }

    return [Math.round(ro * 255), Math.round(go * 255), Math.round(bo * 255)]
}

export function extractDominantColor(imageUrl: string): Promise<string | null> {
    return new Promise((resolve) => {
        const img = new Image()
        img.crossOrigin = 'anonymous'
        img.onload = () => {
            const canvas = document.createElement('canvas')
            const size = 64
            canvas.width = size
            canvas.height = size
            const ctx = canvas.getContext('2d')
            if (!ctx) { resolve(null); return }

            ctx.drawImage(img, 0, 0, size, size)
            const data = ctx.getImageData(0, 0, size, size).data

            const buckets = new Map<string, { r: number; g: number; b: number; count: number }>()

            for (let i = 0; i < data.length; i += 4) {
                const r = data[i]
                const g = data[i + 1]
                const b = data[i + 2]

                const brightness = (r + g + b) / 3
                if (brightness < 30 || brightness > 230) continue

                const max = Math.max(r, g, b)
                const min = Math.min(r, g, b)
                if (max - min < 30) continue

                const qr = Math.round(r / 32) * 32
                const qg = Math.round(g / 32) * 32
                const qb = Math.round(b / 32) * 32
                const key = `${qr},${qg},${qb}`

                const bucket = buckets.get(key)
                if (bucket) {
                    bucket.r += r
                    bucket.g += g
                    bucket.b += b
                    bucket.count++
                } else {
                    buckets.set(key, { r, g, b, count: 1})
                }
            }

            if (buckets.size === 0) { resolve(null); return }

            let best = { r: 0, g: 0, b: 0, count: 0 }
            for (const bucket of buckets.values()) {
                if (bucket.count > best.count) best = bucket
            }

            const avgR = Math.round(best.r / best.count)
            const avgG = Math.round(best.g / best.count)
            const avgB = Math.round(best.b / best.count)
            const [r, g, b] = ensureMinLightness(complementary(avgR, avgG, avgB), 0.6)
            resolve(`#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`)
        }
        img.onerror = () => resolve(null)
        img.src = imageUrl
    })
}