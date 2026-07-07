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
    optimizer?: boolean
    sleepSchedule?: sleepSchedule,
    gameCache?: Record<string, GameCache>;
    crushRpc?: boolean;
    interactive?: interactiveAPI
    disableSystemTray?: boolean
    disableLaunchAtStartUp?: boolean
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

export type CloudConfig = {
    authToken?: string
    automaticallyCloud: boolean
    lastSyncHash?: string
    syncPassword?: string
}

/**
 * Universal, bootstrapper-agnostic cloud config schema.
 * 
 * This part is wrote by claude.ai
 * 
 * Every Roblox bootstrapper (Crush, Bloxstrap, Froststrap, Funkstrap, Voidstrap,
 * AppleBlox, ...) maps its own config to and from this shape, so they can all
 * share a single cloud blob without overwriting each other's data.
 *
 * Rules for consumers:
 *  - Every field is OPTIONAL. Only write the fields you actually understand.
 *  - When writing back, MERGE onto the existing cloud object — never replace it
 *    wholesale, or you'll wipe settings owned by other bootstrappers.
 *  - Preserve unknown keys and the entire `vendor` map of apps that aren't you.
 *  - Check `schemaVersion` before reading; ignore versions you don't support.
 */
export type CloudUniversalConfig = {
    /** Schema version. Bump only on breaking changes. */
    schemaVersion: 1

    /** When this blob was last written, ISO-8601. Used for conflict resolution. */
    updatedAt?: string

    /**
     * Roblox FastFlag overrides (FFlag/DFFlag/FInt/...). The single most
     * portable piece of data — identical meaning across every bootstrapper.
     */
    fastFlags?: Record<string, string | number | boolean>

    integrations?: {
        /** Discord Rich Presence. */
        discordRichPresence?: {
            enabled?: boolean
            /** Show the logged-in Roblox account (ShowAccountOnRichPresence / displayAccount). */
            showAccount?: boolean
            /** Allow others to join via RPC (!HideRPCButtons / letJoin). */
            allowJoining?: boolean
            /** Show current server details (ShowServerDetails). */
            showServerDetails?: boolean
        }

        /** Activity / play-session tracking (EnableActivityTracking / activityWatching). */
        activityTracking?: {
            enabled?: boolean
        }

        /** Roblox crash handler behaviour. */
        crashHandler?: {
            /** Auto-close the crash handler (AutoCloseCrashHandler / closeCrashHandler). */
            autoClose?: boolean
            /** Disable the crash handler entirely (DisableCrash). */
            disable?: boolean
        }

        /** Update behaviour. */
        updates?: {
            checkForUpdates?: boolean
            backgroundUpdates?: boolean
            /** Keep Roblox itself updated (UpdateRoblox). */
            updateRoblox?: boolean
        }

        /** Server selection / matchmaking. */
        matchmaking?: {
            /** Region-biased matchmaking (EnableBetterMatchmaking). */
            betterMatchmaking?: boolean
            /** Notify which region a server is in (serverLocationNotifier). */
            serverRegionNotifier?: boolean
        }

        /** Priority class for the Roblox process. */
        processPriority?:
        | 'below_normal'
        | 'normal'
        | 'above_normal'
        | 'high'
        | 'realtime'

        /** Window control / interaction permissions (funkstrap-style controls / crush interactive). */
        windowControl?: {
            enabled?: boolean
            allowMove?: boolean
            allowTitleChange?: boolean
            allowTransparency?: boolean
        }
    }

    /**
     * Roblox install / channel PREFERENCES (not device state). Installed paths,
     * the active install on this machine, and transient flags like
     * "force reinstall" are device-local and must NOT go here — keep those in
     * `vendor` or out of the cloud entirely.
     */
    installation?: {
        /** Pinned Roblox version GUID, or omit to follow the latest. */
        version?: string
        /** Don't auto-update Roblox past the pinned version (crush dontUpdate). */
        pinVersion?: boolean
        /** Use the VNG (Vietnam) Roblox channel (crush vng). */
        vngChannel?: boolean
        /** How many Roblox instances may run in parallel (crush parallel). */
        parallel?: number
    }

    /** Preferred / best server region the launcher should target (crush bestRegion). */
    bestRegion?: string

    /** Whether FastFlag overrides are applied at all (crush useFlag). */
    fastFlagsEnabled?: boolean

    /**
     * Per-bootstrapper private settings that don't map to the universal shape.
     * Keyed by a stable lowercase bootstrapper id (e.g. "crush", "bloxstrap").
     * A bootstrapper owns only its own key and MUST leave the others untouched.
     */
    vendor?: Record<string, unknown>
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
    cloudConfig: CloudConfig
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