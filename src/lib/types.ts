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

// FastFlag Types
export type FlagType = 'bool' | 'int' | 'string'

// Theme XML Parser Types
export type Theme = 'Dark' | 'Light'
export type HAlign = 'Left' | 'Center' | 'Right' | 'Stretch'
export type VAlign = 'Top' | 'Center' | 'Bottom' | 'Stretch'
export type Visibility = 'Visible' | 'Hidden' | 'Collapsed'

export interface Margin {
    top: number
    right: number
    bottom: number
    left: number
}

export interface BaseElement {
    type: string
    name?: string
    hAlign?: HAlign
    vAlign?: VAlign
    margin?: Margin
    opacity?: number
    zIndex?: number
    visibility?: Visibility
    width?: number
    height?: number
    props: Record<string, any>
}

export type BootstrapElement = BaseElement

export interface BootstrapConfig {
    version: number
    height: number
    width: number
    ignoreTitleBarInset: boolean
    theme: Theme
    margin?: Margin
    windowCornerPreference?: string
    elements: BootstrapElement[]
}

// Theme Store Types
export interface ThemeState {
    themeName: string
    config?: BootstrapConfig
    customHtml?: string
    isHtmlTheme: boolean
    assetMap: Record<string, string>
}

// Theme Loader Types
export interface LoadResult {
    state: ThemeState
    themeName: string
    destDir: string
    missing: string[]
}
