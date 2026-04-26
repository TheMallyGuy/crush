export type Server = {
  server_id: string;
  ip_address: string;
  playing: number;
  max_players: number;
}

export type Result = {
  success: boolean;
  regionTried: string[];
  servers: Server[];
}

export type RoValra = {
    joinServerForYouValue: boolean
}

export type DiscordRpc = {
    enable : boolean,
    displayAccount: boolean,
    letJoin: boolean
}

export type Integrations = {
    discordRpc: DiscordRpc
    serverLocationNotifier: boolean
    roValra : RoValra
    gameCache?: Record<string, GameCache>;
    crushRpc?: boolean;
}

export type BuildInfo = {
    hash: string,
    build_date: string,
    version: string
}

export type RawEntry = {
    place_id: number;
    instance_id?: string;
    timestamp: string;
};

export type GameCache = {
    universeId: number | null;
    name: string;
    imageUrl: string | null;
    cachedAt: string;
};

export type Installation = {
    version: string,
    forceReinstall: boolean,
    dontUpdate: boolean
}

export type Config = {
    installation: Installation
    FirstLaunch: string
    bestRegion: string
    integrations: Integrations
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

export type Versions = {
    versions: string[]
}

export type RobloxLaunchData = {
    raw: Record<string, string>;

    launchmode?: string;
    gameinfo?: string;
    launchtime?: number;

    placelauncherurl?: string;
    request?: string;

    placeId?: number | null;
    userId?: number | null;
    joinAttemptId?: string | null;
};

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

export type AppType = 'player' | 'studio'



export type BoostrapConfigs = { // bloxstrap, froststrap
    // general forks (from bloxstrap)
    CheckForUpdates: boolean,
    BackgroundUpdatesEnabled: boolean,
    UseDiscordRichPresence: boolean,
    ShowServerDetails: boolean,
    ShowAccountOnRichPresence: boolean, // show account
    HideRPCButtons: boolean, // allow joining server

    // frostrap only
    EnableBetterMatchmaking: boolean,
    ShowUsingFroststrapRPC: boolean,

    UpdateRoblox : boolean, // frost & void???

    // voidstrap only
    VoidRPC: boolean,
}