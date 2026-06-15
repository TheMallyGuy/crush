import type {
    CloudUniversalConfig,
    Integrations,
    Installation,
    PriorityClass,
} from '$lib/types'

export const CRUSH_VENDOR_ID = 'crush'

type UniversalIntegrations = NonNullable<CloudUniversalConfig['integrations']>
type UniversalPriority = NonNullable<UniversalIntegrations['processPriority']>

const PRIORITY_TO_UNIVERSAL: Record<PriorityClass, UniversalPriority> = {
    BELOW_NORMAL_PRIORITY_CLASS: 'below_normal',
    NORMAL_PRIORITY_CLASS: 'normal',
    ABOVE_NORMAL_PRIORITY_CLASS: 'above_normal',
    HIGH_PRIORITY_CLASS: 'high',
    REALTIME_PRIORITY_CLASS: 'realtime',
}

const PRIORITY_FROM_UNIVERSAL: Record<UniversalPriority, PriorityClass> = {
    below_normal: 'BELOW_NORMAL_PRIORITY_CLASS',
    normal: 'NORMAL_PRIORITY_CLASS',
    above_normal: 'ABOVE_NORMAL_PRIORITY_CLASS',
    high: 'HIGH_PRIORITY_CLASS',
    realtime: 'REALTIME_PRIORITY_CLASS',
}


export type CrushSnapshot = {
    useFlag?: boolean
    bestRegion?: string
    installation?: Installation
    integrations?: Integrations
    lockedIn?: boolean
    redRings?: boolean
    fastFlags?: Record<string, string>
}

type CrushVendor = {
    integrations?: Integrations
    lockedIn?: boolean
    redRings?: boolean
}

export function crushToUniversal(s: CrushSnapshot): CloudUniversalConfig {
    const ci = s.integrations
    const integrations: UniversalIntegrations = {}

    if (ci?.discordRpc) {
        integrations.discordRichPresence = {
            enabled: ci.discordRpc.enable,
            showAccount: ci.discordRpc.displayAccount,
            allowJoining: ci.discordRpc.letJoin,
        }
    }
    if (ci?.activityWatching !== undefined) {
        integrations.activityTracking = { enabled: ci.activityWatching }
    }
    if (ci?.closeCrashHandler !== undefined) {
        integrations.crashHandler = { autoClose: ci.closeCrashHandler }
    }
    if (ci?.serverLocationNotifier !== undefined) {
        integrations.matchmaking = {
            serverRegionNotifier: ci.serverLocationNotifier,
        }
    }
    if (ci?.priority) {
        integrations.processPriority = PRIORITY_TO_UNIVERSAL[ci.priority]
    }
    if (ci?.interactive) {
        integrations.windowControl = {
            enabled: ci.interactive.enable,
            allowMove: ci.interactive.scopes.moveWindow,
            allowTitleChange: ci.interactive.scopes.setTitle,
            allowTransparency: ci.interactive.scopes.transparencyScopes.enabled,
        }
    }

    const vendor: CrushVendor = {
        integrations: s.integrations,
        lockedIn: s.lockedIn,
        redRings: s.redRings,
    }

    return {
        schemaVersion: 1,
        fastFlagsEnabled: s.useFlag,
        fastFlags: s.fastFlags,
        bestRegion: s.bestRegion || undefined,
        installation: s.installation
            ? {
                version: s.installation.version || undefined,
                pinVersion: s.installation.dontUpdate,
                vngChannel: s.installation.vng,
                parallel: s.installation.parallel,
            }
            : undefined,
        integrations: Object.keys(integrations).length
            ? integrations
            : undefined,
        vendor: { [CRUSH_VENDOR_ID]: vendor },
    }
}


export function universalToCrush(
    u: CloudUniversalConfig,
    current: CrushSnapshot
): CrushSnapshot {
    const vendor = (u.vendor?.[CRUSH_VENDOR_ID] as CrushVendor | undefined) ?? {}

    const integrations: Integrations = {
        ...(current.integrations ?? {}),
        ...(vendor.integrations ?? {}),
    }

    const i = u.integrations
    if (i?.discordRichPresence) {
        const d = integrations.discordRpc
        integrations.discordRpc = {
            enable: i.discordRichPresence.enabled ?? d?.enable ?? false,
            displayAccount:
                i.discordRichPresence.showAccount ?? d?.displayAccount ?? false,
            letJoin:
                i.discordRichPresence.allowJoining ?? d?.letJoin ?? false,
        }
    }
    if (i?.activityTracking?.enabled !== undefined) {
        integrations.activityWatching = i.activityTracking.enabled
    }
    if (i?.crashHandler?.autoClose !== undefined) {
        integrations.closeCrashHandler = i.crashHandler.autoClose
    }
    if (i?.matchmaking?.serverRegionNotifier !== undefined) {
        integrations.serverLocationNotifier = i.matchmaking.serverRegionNotifier
    }
    if (i?.processPriority) {
        integrations.priority = PRIORITY_FROM_UNIVERSAL[i.processPriority]
    }
    if (i?.windowControl && integrations.interactive) {
        const w = i.windowControl
        const it = integrations.interactive
        integrations.interactive = {
            ...it,
            enable: w.enabled ?? it.enable,
            scopes: {
                ...it.scopes,
                moveWindow: w.allowMove ?? it.scopes.moveWindow,
                setTitle: w.allowTitleChange ?? it.scopes.setTitle,
                transparencyScopes: {
                    ...it.scopes.transparencyScopes,
                    enabled:
                        w.allowTransparency ??
                        it.scopes.transparencyScopes.enabled,
                },
            },
        }
    }

    const installation: Installation | undefined = u.installation
        ? {
            version: u.installation.version ?? current.installation?.version ?? '',
            dontUpdate:
                u.installation.pinVersion ??
                current.installation?.dontUpdate ??
                false,
            vng: u.installation.vngChannel ?? current.installation?.vng ?? false,
            parallel:
                u.installation.parallel ?? current.installation?.parallel ?? 1,
            forceReinstall: current.installation?.forceReinstall ?? false,
        }
        : current.installation

    return {
        useFlag: u.fastFlagsEnabled ?? current.useFlag,
        bestRegion: u.bestRegion ?? current.bestRegion,
        installation,
        integrations,
        lockedIn: vendor.lockedIn ?? current.lockedIn,
        redRings: vendor.redRings ?? current.redRings,
        fastFlags: coerceFlags(u.fastFlags) ?? current.fastFlags,
    }
}


export function mergeUniversal(
    base: CloudUniversalConfig | null,
    contribution: CloudUniversalConfig
): CloudUniversalConfig {
    return {
        ...(base ?? { schemaVersion: 1 }),
        ...contribution,
        schemaVersion: 1,
        updatedAt: new Date().toISOString(),
        integrations: {
            ...(base?.integrations ?? {}),
            ...(contribution.integrations ?? {}),
        },
        vendor: {
            ...(base?.vendor ?? {}),
            ...(contribution.vendor ?? {}),
        },
    }
}

function coerceFlags(
    flags: Record<string, string | number | boolean> | undefined
): Record<string, string> | undefined {
    if (!flags) return undefined
    return Object.fromEntries(
        Object.entries(flags).map(([k, v]) => [k, String(v)])
    )
}
