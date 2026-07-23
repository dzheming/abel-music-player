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

            const r = Math.round(best.r / best.count)
            const g = Math.round(best.g / best.count)
            const b = Math.round(best.b / best.count)
            resolve(`#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`)
        }
        img.onerror = () => resolve(null)
        img.src = imageUrl
    })
}