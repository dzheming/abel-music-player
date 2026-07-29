export function adjustBrightness(hex: string, amount: number): string {
    const num = parseInt(hex.replace('#', ''), 16)
    const r = Math.min(255, Math.max(0, ((num >> 16) & 0xff) + amount))
    const g = Math.min(255, Math.max(0, ((num >> 8) & 0xff) + amount))
    const b = Math.min(255, Math.max(0, (num & 0xff) + amount))
    return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`
}