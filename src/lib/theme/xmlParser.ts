import type {
    Theme,
    HAlign,
    VAlign,
    Visibility,
    Margin,
    BootstrapElement,
    BootstrapConfig,
} from '$lib/types'

// ─── Primitive parsers ────────────────────────────────────────────────────────

const attr = (el: Element, name: string): string | undefined =>
    el.getAttribute(name) ?? undefined

function parseMargin(raw: string | undefined): Margin | undefined {
    if (!raw) return undefined
    const p = raw.split(',').map((s) => parseFloat(s.trim()))
    if (p.length === 4) return { left: p[0], top: p[1], right: p[2], bottom: p[3] }
    if (p.length === 2) return { left: p[0], top: p[1], right: p[0], bottom: p[1] }
    if (p.length === 1 && !isNaN(p[0]))
        return { left: p[0], top: p[0], right: p[0], bottom: p[0] }
    return undefined
}

const parseNum = (v?: string): number | undefined =>
    !v || v === 'Auto' || v === '*' ? undefined : isNaN(+v) ? undefined : +v

const parseBool = (v?: string): boolean | undefined =>
    v === undefined ? undefined : v.toLowerCase() === 'true'

// ─── WPF Binding resolution ───────────────────────────────────────────────────

// Text bindings → become el.name so the renderer can inject live values
const TEXT_BINDINGS: Record<string, string> = {
    Message: 'StatusText',
    VersionText: 'VersionText',
}

// Non-text bindings → stored as __marker__ strings in props
const PROP_BINDINGS: Record<string, string> = {
    ByfronLogoLocation: '__logo__',
    Background: '__bg__',
    Foreground: '__fg__',
    IconColor: '__icon__',
    Fill: '__icon__',
    CancelButtonVisibility: '__cancelVisibility__',
    VersionTextVisibility: '__versionVisibility__',
    ProgressValue: '__progressValue__',
    ProgressMaximum: '__progressMax__',
    ProgressIndeterminate: '__progressIndeterminate__',
}

function resolveBinding(value: string): string | undefined {
    const m = value.match(/^\{Binding\s+(?:Path=)?([^,}\s]+)/)
    if (!m) return undefined
    const name = m[1].trim()
    return TEXT_BINDINGS[name] ?? PROP_BINDINGS[name] ?? undefined
}

function resolveAttrValue(value: string | undefined): string | undefined {
    if (!value) return undefined
    if (value.startsWith('{StaticResource')) return undefined // can't resolve static resources
    if (value.startsWith('{Binding')) return resolveBinding(value)
    if (value.startsWith('{')) return undefined // unknown markup extension
    return value
}

// ─── Tag classification ───────────────────────────────────────────────────────

const CONTAINER_TAGS = new Set([
    'Border',
    'Grid',
    'Canvas',
    'StackPanel',
    'DockPanel',
    'WrapPanel',
    'ScrollViewer',
    'Viewbox',
    'GroupBox',
])

function stripNs(tag: string): string {
    return tag.includes(':') ? tag.split(':').slice(1).join(':') : tag
}

function shouldSkip(tag: string): boolean {
    // Property elements (Tag.Property pattern) – handled inside their owner
    if (tag.includes('.')) return true
    const skipped = new Set([
        'RowDefinition',
        'ColumnDefinition',
        'TaskbarItemInfo',
        'Window', // top-level window node itself, we only parse its children
    ])
    return skipped.has(tag)
}

// ─── SVG path extraction ──────────────────────────────────────────────────────

function extractSvgFigures(node: Element): string | undefined {
    // Direct attribute (simple format)
    const direct = attr(node, 'Figures')
    if (direct) return direct

    // Child <Path.Data><PathGeometry Figures="..."/></Path.Data>
    for (const child of Array.from(node.childNodes)) {
        const el = child as Element
        if (el.nodeType !== 1) continue
        if (stripNs(el.tagName) === 'Path.Data') {
            for (const gc of Array.from(el.childNodes)) {
                const geo = gc as Element
                if (geo.nodeType !== 1) continue
                if (stripNs(geo.tagName) === 'PathGeometry') {
                    return attr(geo, 'Figures')
                }
            }
        }
    }
    return undefined
}

// ─── Walk context ─────────────────────────────────────────────────────────────

interface WalkCtx {
    /** Inherited alignment defaults from parent containers */
    defaultHAlign: HAlign
    defaultVAlign: VAlign
    /** Accumulated pixel offsets injected into child margins */
    extraTopOffset: number
    extraBottomOffset: number
    /** Whether we're inside a WPF Grid cell (affects default alignment) */
    inGrid: boolean
}

const ROOT_CTX: WalkCtx = {
    defaultHAlign: 'Left',
    defaultVAlign: 'Top',
    extraTopOffset: 0,
    extraBottomOffset: 0,
    inGrid: false,
}

// ─── Element parser ───────────────────────────────────────────────────────────

function parseElement(node: Element, ctx: WalkCtx): BootstrapElement {
    const tag = stripNs(node.tagName)

    const explicitH = attr(node, 'HorizontalAlignment') || attr(node, 'HAlign')
    const explicitV = attr(node, 'VerticalAlignment') || attr(node, 'VAlign')

    const width = parseNum(attr(node, 'Width'))
    const height = parseNum(attr(node, 'Height'))

    // WPF Grid: no explicit alignment + explicit size → element centers in its cell
    let hAlign = (explicitH ?? (ctx.inGrid && width != null ? 'Center' : ctx.defaultHAlign)) as HAlign
    let vAlign = (explicitV ?? (ctx.inGrid && height != null ? 'Center' : ctx.defaultVAlign)) as VAlign

    // Merge raw margin with accumulated context offsets
    const rawMargin = parseMargin(attr(node, 'Margin'))
    const mergedMargin: Margin = {
        left: rawMargin?.left ?? 0,
        top: (rawMargin?.top ?? 0) + ctx.extraTopOffset,
        right: rawMargin?.right ?? 0,
        bottom: (rawMargin?.bottom ?? 0) + ctx.extraBottomOffset,
    }
    const margin =
        mergedMargin.left || mergedMargin.top || mergedMargin.right || mergedMargin.bottom
            ? mergedMargin
            : undefined

    const el: BootstrapElement = {
        type: tag,
        name: attr(node, 'Name') ?? attr(node, 'x:Name'),
        hAlign,
        vAlign,
        margin,
        opacity: parseNum(attr(node, 'Opacity')),
        zIndex:
            parseNum(attr(node, 'Panel.ZIndex')) ?? parseNum(attr(node, 'ZIndex')),
        visibility: attr(node, 'Visibility') as Visibility,
        width,
        height,
        props: {},
    }

    // Copy all attributes into props (resolve bindings)
    for (let j = 0; j < node.attributes.length; j++) {
        const a = node.attributes[j]
        const resolved = resolveAttrValue(a.value)
        if (resolved !== undefined) el.props[a.name] = resolved
    }

    // ── Type-specific extras ──────────────────────────────────────────────────

    if (tag === 'Path') {
        const figures = extractSvgFigures(node)
        if (figures) el.props.svgFigures = figures
        el.props.svgFillRule =
            attr(node, 'FillRule') === 'NonZero' ? 'nonzero' : 'evenodd'
        const rawFill = attr(node, 'Fill')
        el.props.svgFill = resolveAttrValue(rawFill) ?? '__icon__'
        el.props.svgStretch = attr(node, 'Stretch') ?? 'Fill'
    }

    if (tag === 'Image') {
        const src = resolveAttrValue(attr(node, 'Source'))
        el.props.source = src ?? ''
    }

    if (tag === 'TextBlock') {
        const rawText = attr(node, 'Text')
        const resolved = resolveAttrValue(rawText)
        if (resolved) {
            el.props.Text = resolved
            // If it resolved to a live text binding name, use it as el.name
            // so the renderer can inject textValues[el.name]
            const bindMatch = rawText?.match(/^\{Binding\s+(?:Path=)?([^,}\s]+)/)
            if (bindMatch) {
                const mapped = TEXT_BINDINGS[bindMatch[1].trim()]
                if (mapped) el.name = mapped
            }
        }
        el.props.textAlign = attr(node, 'TextAlignment')
        el.props.fontWeight = attr(node, 'FontWeight')
        el.props.fontFamily = resolveAttrValue(attr(node, 'FontFamily'))
    }

    // Collect theme asset sources
    const assetAttrs = ['Source', 'ImageSource', 'Background']
    for (const a of assetAttrs) {
        const val = resolveAttrValue(attr(node, a))
        if (
            val &&
            !val.startsWith('__') &&
            (val.startsWith('theme://') ||
                (!val.includes('://') &&
                    (val.endsWith('.png') ||
                        val.endsWith('.jpg') ||
                        val.endsWith('.gif'))))
        ) {
            el.props.source = val
            break
        }
    }

    return el
}

// ─── Grid row layout ──────────────────────────────────────────────────────────

function getRowHeights(grid: Element): number[] {
    for (const child of Array.from(grid.childNodes)) {
        const el = child as Element
        if (el.nodeType !== 1) continue
        if (stripNs(el.tagName) === 'Grid.RowDefinitions') {
            return Array.from(el.childNodes)
                .filter((n): n is Element => (n as Element).nodeType === 1)
                .map((rd) => {
                    const h = (rd as Element).getAttribute('Height')
                    // Explicit numeric height; 0 = Auto (estimated later)
                    return h && !isNaN(+h) ? +h : 0
                })
        }
    }
    return []
}

function collectGridChildren(grid: Element, parentCtx: WalkCtx): BootstrapElement[] {
    const gridVAlign =
        ((attr(grid, 'VerticalAlignment') || attr(grid, 'VAlign')) as VAlign) ??
        parentCtx.defaultVAlign
    const gridHAlign =
        ((attr(grid, 'HorizontalAlignment') || attr(grid, 'HAlign')) as HAlign) ??
        parentCtx.defaultHAlign

    const gridMargin = parseMargin(attr(grid, 'Margin'))
    const baseTopOffset = parentCtx.extraTopOffset + (gridMargin?.top ?? 0)
    const baseBottomOffset = parentCtx.extraBottomOffset + (gridMargin?.bottom ?? 0)

    const rowHeights = getRowHeights(grid)
    const hasRows = rowHeights.length > 0

    const baseCtx: WalkCtx = {
        defaultHAlign: gridHAlign,
        defaultVAlign: gridVAlign,
        extraTopOffset: baseTopOffset,
        extraBottomOffset: baseBottomOffset,
        inGrid: true,
    }

    if (!hasRows) return collectElements(grid, baseCtx)

    // Collect visual children and group by Grid.Row
    type RowEntry = { node: Element; row: number }
    const entries: RowEntry[] = []

    for (const child of Array.from(grid.childNodes)) {
        const node = child as Element
        if (node.nodeType !== 1) continue
        const tag = stripNs(node.tagName)
        if (shouldSkip(tag)) continue
        const row = parseInt(node.getAttribute('Grid.Row') ?? '0')
        entries.push({ node, row })
    }

    // Fill in auto row heights from element content
    const heights = [...rowHeights]
    for (const { node, row } of entries) {
        if (heights[row] === 0) {
            const h = parseNum(attr(node, 'Height'))
            const fontSize = parseNum(attr(node, 'FontSize'))
            heights[row] = h ?? (fontSize ? Math.ceil(fontSize * 1.5) : 30)
        }
    }

    // Cumulative top offsets per row
    const rowTopOffset: number[] = []
    let cum = 0
    for (const h of heights) {
        rowTopOffset.push(cum)
        cum += h
    }
    const totalRowHeight = cum

    const results: BootstrapElement[] = []

    for (const { node, row } of entries) {
        const tag = stripNs(node.tagName)
        const rowTop = rowTopOffset[row] ?? 0
        const rowH = heights[row] ?? 30

        let childCtx: WalkCtx

        if (gridVAlign === 'Bottom') {
            // Rows stack downward inside the grid. Compute each row's distance from the
            // grid's bottom edge, then add the grid's own bottom offset.
            const rowBottomFromGridBottom = totalRowHeight - rowTop - rowH
            childCtx = {
                ...baseCtx,
                extraTopOffset: 0,
                extraBottomOffset: baseBottomOffset + rowBottomFromGridBottom,
            }
        } else {
            childCtx = {
                ...baseCtx,
                extraTopOffset: baseTopOffset + rowTop,
                extraBottomOffset: 0,
            }
        }

        if (CONTAINER_TAGS.has(tag)) {
            results.push(...collectFromContainer(node, childCtx))
        } else {
            results.push(parseElement(node, childCtx))
        }
    }

    return results
}

// ─── Recursive collector ──────────────────────────────────────────────────────

function collectFromContainer(node: Element, ctx: WalkCtx): BootstrapElement[] {
    const tag = stripNs(node.tagName)
    if (tag === 'Grid') return collectGridChildren(node, ctx)

    const margin = parseMargin(attr(node, 'Margin'))
    const childCtx: WalkCtx = {
        defaultHAlign:
            ((attr(node, 'HorizontalAlignment') || attr(node, 'HAlign')) as HAlign) ??
            ctx.defaultHAlign,
        defaultVAlign:
            ((attr(node, 'VerticalAlignment') || attr(node, 'VAlign')) as VAlign) ??
            ctx.defaultVAlign,
        extraTopOffset: ctx.extraTopOffset + (margin?.top ?? 0),
        extraBottomOffset: ctx.extraBottomOffset + (margin?.bottom ?? 0),
        inGrid: ctx.inGrid,
    }
    return collectElements(node, childCtx)
}

function collectElements(root: Element, ctx: WalkCtx): BootstrapElement[] {
    const results: BootstrapElement[] = []

    for (const child of Array.from(root.childNodes)) {
        const node = child as Element
        if (node.nodeType !== 1) continue

        const tag = stripNs(node.tagName)
        if (shouldSkip(tag)) continue

        if (CONTAINER_TAGS.has(tag)) {
            results.push(...collectFromContainer(node, ctx))
        } else {
            results.push(parseElement(node, ctx))
        }
    }

    return results
}

// ─── Public entry point ───────────────────────────────────────────────────────

export function parseXml(xml: string): BootstrapConfig {
    const doc = new window.DOMParser().parseFromString(xml, 'text/xml')
    const root = doc.documentElement
    const rootTag = stripNs(root.tagName)

    const parseErr = root.querySelector('parsererror')
    if (parseErr) {
        throw new Error(`XML parse error: ${parseErr.textContent?.trim()}`)
    }

    const config: BootstrapConfig = {
        version: parseInt(attr(root, 'Version') ?? '1'),
        // WPF Window uses Width/Height; custom format may too
        width: parseInt(attr(root, 'Width') ?? '600'),
        height: parseInt(attr(root, 'Height') ?? '400'),
        ignoreTitleBarInset: parseBool(attr(root, 'IgnoreTitleBarInset')) ?? false,
        theme: (attr(root, 'Theme') as Theme) ?? 'Dark',
        margin: parseMargin(attr(root, 'Margin')),
        windowCornerPreference: attr(root, 'WindowCornerPreference'),
        elements: [],
    }

    // For WPF Window elements, recurse into children rather than treating
    // the Window itself as a visual element
    const startNode =
        rootTag === 'Window' || rootTag.endsWith(':Window') ? root : root

    config.elements = collectElements(startNode, ROOT_CTX)
    config.elements.sort((a, b) => (a.zIndex ?? 0) - (b.zIndex ?? 0))

    return config
}
