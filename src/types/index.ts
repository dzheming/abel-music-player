export interface Track {
    path: string
    fileName: string
    title?: string
    artist?: string
    album?: string
    duration?: number
    coverUrl?: string
    trackNumber?: number
}

export interface RawTrack {
    path: string
    file_name: string
    title: string | null
    artist: string | null
    album: string | null
    duration: number
    cover?: string | null
    track_number: number | null
}

export interface RawPlaylistTrack extends RawTrack {
    position: number
}

export enum LoopMode {
    None = 'none',
    RepeatOne = 'repeat-one',
    RepeatAll = 'repeat-all',
}

export enum ThemeMode {
    Light = 'light',
    Dark = 'dark',
}

export interface PlayList {
    id: number
    name: string
    track_count: number
    created_at: string
}

export interface ArtistGroup {
    artist: string
    track_count: number
}

export interface AlbumGroup {
    album: string
    artist?: string
    track_count: number
}

export interface LibraryFolder {
    id: number
    path: string
    name: string
    parent_path: string | null
    is_root: boolean
    excluded: boolean
    audio_count: number
}

export interface LibraryFolderNode {
    path: string
    name: string
    audio_count: number
    children: LibraryFolderNode[]
}

export function toTrack(raw: RawTrack): Track {
    return {
        path: raw.path,
        fileName: raw.file_name,
        title: raw.title || undefined,
        artist: raw.artist || undefined,
        album: raw.album || undefined,
        duration: raw.duration,
        coverUrl: raw.cover || undefined,
        trackNumber: raw.track_number || undefined,
    }
}
