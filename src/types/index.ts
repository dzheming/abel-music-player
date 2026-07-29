export interface RawMetadata {
    path: string
    file_name: string
    title: string | null
    artist: string | null
    album: string | null
    duration: number
    cover: string | null
    track_number: number | null
}

export interface AudioFile {
    path: string
    fileName: string
    title?: string
    artist?: string
    album?: string
    duration?: number
    coverUrl?: string
    trackNumber?: number
}

export interface MusicFolder {
    path: string
    name: string
}

export interface FolderNode {
    name: string
    path: string
    children: FolderNode[]
    audio_count: number
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

export interface PlayListTrack {
    path: string
    file_name: string
    title?: string
    artist?: string
    album?: string
    duration: number
    position: number
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

export interface CachedTrackData {
    path: string
    file_name: string
    title: string | null
    artist: string | null
    album: string | null
    duration: number
    track_number: number | null
}