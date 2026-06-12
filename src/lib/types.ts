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
    enable: boolean,
    displayAccount: boolean,
    letJoin: boolean
}

export type gameScopes = {
    allowAllGames: boolean,
    allowedGames: [number],
}

export type transparencyScopes = {
    enabled: boolean,
    maxTransparency: number,
    minTransparency: number
}

export type interactiveAPIScopes = {
    transparencyScopes: transparencyScopes
    minimize: boolean
    focus: boolean
    moveWindow: boolean
    maximize: boolean
    restore: boolean
    setTitle: boolean
    setBorderless: boolean
    // gameScopes : gameScopes save this for later xd
}

export type interactiveAPI = {
    enable: boolean,
    scopes: interactiveAPIScopes
}

export type PriorityClass =
    | "BELOW_NORMAL_PRIORITY_CLASS"
    | "NORMAL_PRIORITY_CLASS"
    | "ABOVE_NORMAL_PRIORITY_CLASS"
    | "HIGH_PRIORITY_CLASS"
    | "REALTIME_PRIORITY_CLASS";

export type sleepSchedule = {
    visible: boolean,
    enabled: boolean
}

export type Integrations = {
    closeCrashHandler?: boolean
    activityWatching?: boolean
    discordRpc?: DiscordRpc
    serverLocationNotifier?: boolean
    priority?: PriorityClass
    roValra?: RoValra
    optimizer?: boolean
    sleepSchedule?: sleepSchedule,
    gameCache?: Record<string, GameCache>;
    crushRpc?: boolean;
    interactive?: interactiveAPI
    swifttunnel?: Swifttunnel
}

export type Swifttunnel = {
    enable: boolean
    enableRouting: boolean
    disconnectWhenRobloxClosed: boolean
    perferedRegion: string
    assetsRouting: boolean
    enableCountryBan: boolean
    enablePartialCountryBan: boolean
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

export type InstallationEntry = {
    id: string
    versionHash: string
    appType: AppType
    installedAt: string
    active?: boolean
    channel: 'global' | 'vng'
}

export type Installation = {
    version: string,
    forceReinstall: boolean,
    dontUpdate: boolean,
    vng: boolean,
    parallel: number
}

export type Config = {
    useFlag: boolean
    installation: Installation
    FirstLaunch: string
    bestRegion: string
    integrations: Integrations
    lockedIn?: boolean
    redRings?: boolean
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
    background?: string
    foreground?: string
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

//
export type ServerInfoFromBackend = {
    region_info: string
    game_id: number,
    server_id: string
}

//

export type BoostrapConfigs = { // bloxstrap, froststrap
    // general forks (from bloxstrap)
    EnableActivityTracking: boolean,
    CheckForUpdates: boolean,
    BackgroundUpdatesEnabled: boolean,
    UseDiscordRichPresence: boolean,
    ShowServerDetails: boolean,
    ShowAccountOnRichPresence: boolean, // show account
    HideRPCButtons: boolean, // allow joining server

    // frostrap only
    EnableBetterMatchmaking: boolean,
    ShowUsingFroststrapRPC: boolean,
    AutoCloseCrashHandler: boolean,

    UpdateRoblox: boolean, // frost & void??? (ps : fish)

    // voidstrap only
    VoidRPC: boolean,
    DisableCrash: boolean,

    // funkstrap
    UseWindowControl: boolean
    MoveWindowAllowed: boolean
    TitleControlAllowed: boolean
    WindowTransparencyAllowed: boolean
}