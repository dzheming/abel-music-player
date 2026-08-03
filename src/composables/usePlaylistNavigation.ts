/**
 * 计算下一首索引。shuffle 模式下随机选一个不等于当前索引的位置；
 * 顺序模式下返回 (current + 1) % length。
 */
export function pickNextIndex(
    currentIndex: number,
    length: number,
    shuffle: boolean,
): number {
    if (length === 0) return -1
    if (shuffle) return pickShuffleIndex(currentIndex, length)
    return (currentIndex + 1) % length
}

/**
 * 计算上一首索引。shuffle 模式下随机选一个不等于当前索引的位置；
 * 顺序模式下返回 (current - 1 + length) % length。
 */
export function pickPrevIndex(
    currentIndex: number,
    length: number,
    shuffle: boolean,
): number {
    if (length === 0) return -1
    if (shuffle) return pickShuffleIndex(currentIndex, length)
    return (currentIndex - 1 + length) % length
}

/** 在 [0, length) 中随机选一个不等于 excludeIndex 的索引 */
function pickShuffleIndex(excludeIndex: number, length: number): number {
    if (length === 1) return 0
    let idx: number
    do {
        idx = Math.floor(Math.random() * length)
    } while (idx === excludeIndex)
    return idx
}
