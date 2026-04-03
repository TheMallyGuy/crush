// @pochita find any types and export it here and update the types import

export type Intergrations = {
    crushRpc: boolean
    serverLocationNotifier: boolean
}

export type BuildInfo = {
    hash: string,
    build_date: string,
}

export type Installation = {
    version: string
}

export type Config = {
    installation: Installation
    FirstLaunch: string
    bestRegion: string
    intergrations: Intergrations
}

export type Mod = {
    id: string
    name: string
    enabled: boolean
}

export type ProgressEvent =
    | { type: 'status'; message: string }
    | { type: 'download'; file: string; done: number; total: number }
    | { type: 'extract'; file: string; done: number; total: number }

export type ProgressCallback = (event: ProgressEvent) => void
