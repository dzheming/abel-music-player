export interface LrcLine {
    time: number
    text: string
}

export function parseLrc(content: string): LrcLine[] {
    const lines: LrcLine[] = []
    const timeRegex = /\[(\d{2}):(\d{2})(?:\.(\d{2,3}))?\]/g

    for (const line of content.split('\n')) {
        const trimmed = line.trim()
        if (!trimmed) continue

        const times: number[] = []
        let match: RegExpExecArray | null
        let lastIndex = 0

        timeRegex.lastIndex = 0
        while((match = timeRegex.exec(trimmed)) !== null) {
            const min = parseInt(match[1], 10)
            const sec = parseInt(match[2], 10)
            const ms = match[3] ? parseInt(match[3].padEnd(3, '0'), 10) : 0
            times.push(min * 60 + sec + ms / 1000)
            lastIndex = timeRegex.lastIndex
        }

        if (times.length === 0) continue

        const text = trimmed.slice(lastIndex).trim()
        for (const time of times) {
            lines.push({ time, text })
        }
    }

    lines.sort((a, b) => a.time - b.time)
    return lines
}

export function findCurrentLine(lines: LrcLine[], currentTime: number): number {
    if (lines.length === 0) return -1
    for (let i = lines.length - 1; i >= 0; i--) {
        if (currentTime >= lines[i].time) return i
    }
    return -1
}