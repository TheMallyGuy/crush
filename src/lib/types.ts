// @pochita find any types and export it here and update the types import

export type Intergrations = {
    crushRpc: boolean
    serverLocationNotifier: boolean
}

export type Config = {
    FirstLaunch: string
    bestRegion: string
    intergrations: Intergrations
}
