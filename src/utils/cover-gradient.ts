function hashString(str: string): number {
    let hash = 0
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i)
        hash = ((hash << 5) - hash) + char
        hash = hash & 0x7fffffff
    }
    return Math.abs(hash)
}

export function generateGradient(title: string, artist?: string): string {
    const seed = title + (artist || '')
    const hash = hashString(seed)

    const hue1 = hash % 360
    const hue2 = (hue1 + 40 + (hash % 60)) % 360
    const sat = 55 + (hash % 25)
    const light = 45 + (hash % 15)

    const angle = (hash % 4) * 90 + 45

    return `linear-gradient(${angle}deg, hsl(${hue1}, ${sat}%, ${light}%), hsl(${hue2}, ${sat}%, ${light}%))`
}