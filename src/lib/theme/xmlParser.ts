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


const TEXT_BINDINGS: Record<string, string> = {
    Message: 'StatusText',
    VersionText: 'VersionText',
}

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
    if (value.startsWith('{StaticResource')) return undefined
    if (value.startsWith('{Binding')) return resolveBinding(value)
    if (value.startsWith('{')) return undefined
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
    if (tag.includes('.')) return true
    return new Set(['RowDefinition', 'ColumnDefinition', 'TaskbarItemInfo', 'Window']).has(tag)
}

// ─── SVG path extraction ──────────────────────────────────────────────────────

function extractSvgFigures(node: Element): string | undefined {
    const direct = attr(node, 'Figures')
    if (direct) return direct
    for (const child of Array.from(node.childNodes)) {
        const el = child as Element
        if (el.nodeType !== 1) continue
        if (stripNs(el.tagName) === 'Path.Data') {
            for (const gc of Array.from(el.childNodes)) {
                const geo = gc as Element
                if (geo.nodeType !== 1) continue
                if (stripNs(geo.tagName) === 'PathGeometry') return attr(geo, 'Figures')
            }
        }
    }
    return undefined
}

// ─── Size estimation (needed for stacking without explicit dimensions) ─────────

function estimateNodeWidth(node: Element): number {
    const explicit = parseNum(attr(node, 'Width'))
    if (explicit != null) return explicit

    const tag = stripNs(node.tagName)
    if (CONTAINER_TAGS.has(tag)) {
        const horiz = (attr(node, 'Orientation') ?? 'Vertical') === 'Horizontal'
        let sum = 0, max = 0
        for (const child of Array.from(node.childNodes)) {
            const el = child as Element
            if (el.nodeType !== 1 || shouldSkip(stripNs(el.tagName))) continue
            const m = parseMargin(attr(el, 'Margin'))
            const w = estimateNodeWidth(el)
            if (horiz) { sum += (m?.left ?? 0) + w + (m?.right ?? 0) }
            else        { max = Math.max(max, (m?.left ?? 0) + w) }
        }
        return horiz ? sum : max
    }

    // TextBlock: rough character-width estimate
    const text = attr(node, 'Text') ?? ''
    const fontSize = parseNum(attr(node, 'FontSize')) ?? 12
    return Math.ceil(text.length * fontSize * 0.6)
}

function estimateNodeHeight(node: Element): number {
    const explicit = parseNum(attr(node, 'Height'))
    if (explicit != null) return explicit

    const tag = stripNs(node.tagName)
    if (CONTAINER_TAGS.has(tag)) {
        const vert = (attr(node, 'Orientation') ?? 'Vertical') === 'Vertical'
        let sum = 0, max = 0
        for (const child of Array.from(node.childNodes)) {
            const el = child as Element
            if (el.nodeType !== 1 || shouldSkip(stripNs(el.tagName))) continue
            const m = parseMargin(attr(el, 'Margin'))
            const h = estimateNodeHeight(el)
            if (vert) { sum += (m?.top ?? 0) + h + (m?.bottom ?? 0) }
            else       { max = Math.max(max, (m?.top ?? 0) + h) }
        }
        return vert ? sum : max
    }

    const fontSize = parseNum(attr(node, 'FontSize'))
    if (fontSize) return Math.ceil(fontSize * 1.5)
    if (tag === 'TextBlock') return 20
    if (tag === 'ProgressBar') return 20
    return 30
}

// ─── Walk context ─────────────────────────────────────────────────────────────

interface WalkCtx {
    defaultHAlign: HAlign
    defaultVAlign: VAlign
    /** Accumulated offsets merged into child element margins */
    extraTopOffset: number
    extraBottomOffset: number
    extraLeftOffset: number
    /** Parent container dimensions — used for centering / bottom-align calculations */
    containerWidth: number
    containerHeight: number
    inGrid: boolean
}

function makeRootCtx(w: number, h: number): WalkCtx {
    return {
        defaultHAlign: 'Left',
        defaultVAlign: 'Top',
        extraTopOffset: 0,
        extraBottomOffset: 0,
        extraLeftOffset: 0,
        containerWidth: w,
        containerHeight: h,
        inGrid: false,
    }
}

// ─── Element parser ───────────────────────────────────────────────────────────

function parseElement(node: Element, ctx: WalkCtx): BootstrapElement {
    const tag = stripNs(node.tagName)

    const explicitH = attr(node, 'HorizontalAlignment') || attr(node, 'HAlign')
    const explicitV = attr(node, 'VerticalAlignment')   || attr(node, 'VAlign')
    const width  = parseNum(attr(node, 'Width'))
    const height = parseNum(attr(node, 'Height'))

    // WPF Grid: no explicit alignment + explicit size → center in cell
    const hAlign = (explicitH ?? (ctx.inGrid && width  != null ? 'Center' : ctx.defaultHAlign)) as HAlign
    const vAlign = (explicitV ?? (ctx.inGrid && height != null ? 'Center' : ctx.defaultVAlign)) as VAlign

    // Merge raw margin with accumulated context offsets
    const rawMargin = parseMargin(attr(node, 'Margin'))
    const ml = (rawMargin?.left   ?? 0) + ctx.extraLeftOffset
    const mt = (rawMargin?.top    ?? 0) + ctx.extraTopOffset
    const mr =  rawMargin?.right  ?? 0
    const mb = (rawMargin?.bottom ?? 0) + ctx.extraBottomOffset
    const margin: Margin | undefined =
        (ml !== 0 || mt !== 0 || mr !== 0 || mb !== 0)
            ? { left: ml, top: mt, right: mr, bottom: mb }
            : undefined

    const el: BootstrapElement = {
        type: tag,
        name: attr(node, 'Name') ?? attr(node, 'x:Name'),
        hAlign,
        vAlign,
        margin,
        opacity: parseNum(attr(node, 'Opacity')),
        zIndex: parseNum(attr(node, 'Panel.ZIndex')) ?? parseNum(attr(node, 'ZIndex')),
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
        el.props.svgFillRule = attr(node, 'FillRule') === 'NonZero' ? 'nonzero' : 'evenodd'
        el.props.svgFill = resolveAttrValue(attr(node, 'Fill')) ?? '__icon__'
        el.props.svgStretch = attr(node, 'Stretch') ?? 'Fill'
    }

    if (tag === 'Image') {
        el.props.source = resolveAttrValue(attr(node, 'Source')) ?? ''
    }

    if (tag === 'TextBlock') {
        const rawText = attr(node, 'Text')
        const resolved = resolveAttrValue(rawText)
        if (resolved) {
            el.props.Text = resolved
            const bindMatch = rawText?.match(/^\{Binding\s+(?:Path=)?([^,}\s]+)/)
            if (bindMatch) {
                const mapped = TEXT_BINDINGS[bindMatch[1].trim()]
                if (mapped) el.name = mapped
            }
        }
        el.props.textAlign  = attr(node, 'TextAlignment')
        el.props.fontWeight = attr(node, 'FontWeight')
        el.props.fontFamily = resolveAttrValue(attr(node, 'FontFamily'))
    }

    // Collect theme:// asset sources
    for (const a of ['Source', 'ImageSource', 'Background']) {
        const val = resolveAttrValue(attr(node, a))
        if (val && !val.startsWith('__') && (
            val.startsWith('theme://') ||
            (!val.includes('://') && (val.endsWith('.png') || val.endsWith('.jpg') || val.endsWith('.gif')))
        )) {
            el.props.source = val
            break
        }
    }

    return el
}

// ─── StackPanel layout ────────────────────────────────────────────────────────

function collectStackPanelChildren(panel: Element, parentCtx: WalkCtx): BootstrapElement[] {
    const orientation = attr(panel, 'Orientation') ?? 'Vertical'
    const panelMargin = parseMargin(attr(panel, 'Margin'))
    const panelHAlign = (attr(panel, 'HorizontalAlignment') as HAlign) ?? parentCtx.defaultHAlign
    const panelVAlign = (attr(panel, 'VerticalAlignment')   as VAlign) ?? parentCtx.defaultVAlign

    const pML = panelMargin?.left   ?? 0
    const pMT = panelMargin?.top    ?? 0
    const pMR = panelMargin?.right  ?? 0
    const pMB = panelMargin?.bottom ?? 0

    const totalW = estimateNodeWidth(panel)
    const totalH = estimateNodeHeight(panel)

    // ── Compute the panel's own origin (left / top) in absolute coords ────────

    let originLeft: number
    if (panelHAlign === 'Center') {
        originLeft = parentCtx.extraLeftOffset + (parentCtx.containerWidth - totalW) / 2 + pML
    } else if (panelHAlign === 'Right') {
        originLeft = parentCtx.extraLeftOffset + parentCtx.containerWidth - totalW - pMR
    } else {
        originLeft = parentCtx.extraLeftOffset + pML
    }

    let originTop: number
    if (panelVAlign === 'Bottom') {
        originTop = parentCtx.containerHeight - totalH - pMB
    } else if (panelVAlign === 'Center') {
        originTop = parentCtx.extraTopOffset + (parentCtx.containerHeight - totalH) / 2 + pMT
    } else {
        originTop = parentCtx.extraTopOffset + pMT
    }

    // ── Horizontal stacking ───────────────────────────────────────────────────

    if (orientation === 'Horizontal') {
        let stackLeft = originLeft
        const results: BootstrapElement[] = []

        for (const child of Array.from(panel.childNodes)) {
            const node = child as Element
            if (node.nodeType !== 1) continue
            const tag = stripNs(node.tagName)
            if (shouldSkip(tag)) continue

            const rawM  = parseMargin(attr(node, 'Margin'))
            const rawML = rawM?.left ?? 0
            const rawMR = rawM?.right ?? 0
            const childW = estimateNodeWidth(node)
            const childH = estimateNodeHeight(node)

            const childCtx: WalkCtx = {
                defaultHAlign: 'Left',
                defaultVAlign: 'Top',
                extraLeftOffset: stackLeft,  // parseElement adds rawML on top
                extraTopOffset: originTop,   // parseElement adds rawMT on top
                extraBottomOffset: 0,
                containerWidth: childW,
                containerHeight: childH,
                inGrid: false,
            }

            if (CONTAINER_TAGS.has(tag)) {
                results.push(...collectFromContainer(node, childCtx))
            } else {
                results.push(parseElement(node, childCtx))
            }

            stackLeft += rawML + childW + rawMR
        }

        return results
    }

    // ── Vertical stacking ─────────────────────────────────────────────────────

    let stackTop = originTop
    const results: BootstrapElement[] = []

    for (const child of Array.from(panel.childNodes)) {
        const node = child as Element
        if (node.nodeType !== 1) continue
        const tag = stripNs(node.tagName)
        if (shouldSkip(tag)) continue

        const rawM  = parseMargin(attr(node, 'Margin'))
        const rawMT = rawM?.top    ?? 0
        const rawMB = rawM?.bottom ?? 0
        const childW = estimateNodeWidth(node)
        const childH = estimateNodeHeight(node)

        const childCtx: WalkCtx = {
            defaultHAlign: panelHAlign,
            defaultVAlign: 'Top',
            extraLeftOffset: originLeft,  // pass panel's left to children
            extraTopOffset: stackTop,     // parseElement adds rawMT on top
            extraBottomOffset: 0,
            containerWidth: childW,
            containerHeight: childH,
            inGrid: false,
        }

        if (CONTAINER_TAGS.has(tag)) {
            const children = collectFromContainer(node, childCtx)
            results.push(...children)
            // Advance stack by tallest bottom edge of the nested container
            const bottom = children.reduce((max, c) => {
                const top = c.margin?.top ?? stackTop + rawMT
                return Math.max(max, top + (c.height ?? childH))
            }, stackTop + rawMT)
            stackTop = bottom + rawMB
        } else {
            const el = parseElement(node, childCtx)
            results.push(el)
            const mergedTop = el.margin?.top ?? (stackTop + rawMT)
            stackTop = mergedTop + estimateNodeHeight(node) + rawMB
        }
    }

    return results
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
                    return h && !isNaN(+h) ? +h : 0
                })
        }
    }
    return []
}

function collectGridChildren(grid: Element, parentCtx: WalkCtx): BootstrapElement[] {
    const gridVAlign = ((attr(grid, 'VerticalAlignment')   || attr(grid, 'VAlign')) as VAlign) ?? parentCtx.defaultVAlign
    const gridHAlign = ((attr(grid, 'HorizontalAlignment') || attr(grid, 'HAlign')) as HAlign) ?? parentCtx.defaultHAlign

    const gm = parseMargin(attr(grid, 'Margin'))
    const baseTop    = parentCtx.extraTopOffset    + (gm?.top    ?? 0)
    const baseBottom = parentCtx.extraBottomOffset + (gm?.bottom ?? 0)
    const baseLeft   = parentCtx.extraLeftOffset   + (gm?.left   ?? 0)

    const rowHeights = getRowHeights(grid)
    const hasRows = rowHeights.length > 0

    const baseCtx: WalkCtx = {
        defaultHAlign: gridHAlign,
        defaultVAlign: gridVAlign,
        extraTopOffset: baseTop,
        extraBottomOffset: baseBottom,
        extraLeftOffset: baseLeft,
        containerWidth: parentCtx.containerWidth,
        containerHeight: parentCtx.containerHeight,
        inGrid: true,
    }

    if (!hasRows) return collectElements(grid, baseCtx)

    type RowEntry = { node: Element; row: number }
    const entries: RowEntry[] = []
    for (const child of Array.from(grid.childNodes)) {
        const node = child as Element
        if (node.nodeType !== 1) continue
        const tag = stripNs(node.tagName)
        if (shouldSkip(tag)) continue
        entries.push({ node, row: parseInt(node.getAttribute('Grid.Row') ?? '0') })
    }

    const heights = [...rowHeights]
    for (const { node, row } of entries) {
        if (heights[row] === 0) {
            heights[row] = parseNum(attr(node, 'Height')) ??
                (parseNum(attr(node, 'FontSize')) ? Math.ceil(parseNum(attr(node, 'FontSize'))! * 1.5) : 30)
        }
    }

    const rowTopOffset: number[] = []
    let cum = 0
    for (const h of heights) { rowTopOffset.push(cum); cum += h }
    const totalRowHeight = cum

    const results: BootstrapElement[] = []
    for (const { node, row } of entries) {
        const tag  = stripNs(node.tagName)
        const rowTop = rowTopOffset[row] ?? 0
        const rowH   = heights[row] ?? 30

        let childCtx: WalkCtx
        if (gridVAlign === 'Bottom') {
            const rowBottom = totalRowHeight - rowTop - rowH
            childCtx = { ...baseCtx, extraTopOffset: 0, extraBottomOffset: baseBottom + rowBottom }
        } else {
            childCtx = { ...baseCtx, extraTopOffset: baseTop + rowTop, extraBottomOffset: 0 }
        }

        if (CONTAINER_TAGS.has(tag)) results.push(...collectFromContainer(node, childCtx))
        else                         results.push(parseElement(node, childCtx))
    }

    return results
}

// ─── Recursive collector ──────────────────────────────────────────────────────

function collectFromContainer(node: Element, ctx: WalkCtx): BootstrapElement[] {
    const tag = stripNs(node.tagName)

    if (tag === 'Grid')       return collectGridChildren(node, ctx)
    if (tag === 'StackPanel') return collectStackPanelChildren(node, ctx)

    // Border: render its own background visually AND recurse into children
    if (tag === 'Border') {
        const results: BootstrapElement[] = []
        const hasBg = attr(node, 'Background') || attr(node, 'BorderBrush')
        if (hasBg || attr(node, 'Width') || attr(node, 'Height')) {
            const visualEl = parseElement(node, ctx)
            visualEl.type = 'Border'
            results.push(visualEl)
        }
        const m = parseMargin(attr(node, 'Margin'))
        const childCtx: WalkCtx = {
            defaultHAlign: ((attr(node, 'HorizontalAlignment') || attr(node, 'HAlign')) as HAlign) ?? ctx.defaultHAlign,
            defaultVAlign: ((attr(node, 'VerticalAlignment')   || attr(node, 'VAlign')) as VAlign) ?? ctx.defaultVAlign,
            extraTopOffset: ctx.extraTopOffset + (m?.top ?? 0),
            extraBottomOffset: ctx.extraBottomOffset + (m?.bottom ?? 0),
            extraLeftOffset: ctx.extraLeftOffset + (m?.left ?? 0),
            containerWidth: parseNum(attr(node, 'Width')) ?? ctx.containerWidth,
            containerHeight: parseNum(attr(node, 'Height')) ?? ctx.containerHeight,
            inGrid: ctx.inGrid,
        }
        results.push(...collectElements(node, childCtx))
        return results
    }

    // Generic container (Canvas, DockPanel, etc.)
    const m = parseMargin(attr(node, 'Margin'))
    const childCtx: WalkCtx = {
        defaultHAlign: ((attr(node, 'HorizontalAlignment') || attr(node, 'HAlign')) as HAlign) ?? ctx.defaultHAlign,
        defaultVAlign: ((attr(node, 'VerticalAlignment')   || attr(node, 'VAlign')) as VAlign) ?? ctx.defaultVAlign,
        extraTopOffset: ctx.extraTopOffset + (m?.top ?? 0),
        extraBottomOffset: ctx.extraBottomOffset + (m?.bottom ?? 0),
        extraLeftOffset: ctx.extraLeftOffset + (m?.left ?? 0),
        containerWidth: parseNum(attr(node, 'Width')) ?? ctx.containerWidth,
        containerHeight: parseNum(attr(node, 'Height')) ?? ctx.containerHeight,
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
        if (CONTAINER_TAGS.has(tag)) results.push(...collectFromContainer(node, ctx))
        else                         results.push(parseElement(node, ctx))
    }
    return results
}

// ─── Public entry point ───────────────────────────────────────────────────────

const ROOT_CONTAINER_TAGS = new Set([
    'Window',
    'BloxstrapCustomBootstrapper',
    'BootstrapperStyle',
])

export function parseXml(xml: string): BootstrapConfig {
    const doc = new window.DOMParser().parseFromString(xml, 'text/xml')
    const root = doc.documentElement
    const rootTag = stripNs(root.tagName)

    const parseErr = root.querySelector('parsererror')
    if (parseErr) throw new Error(`XML parse error: ${parseErr.textContent?.trim()}`)

    const rawBg = attr(root, 'Background')
    const rawFg = attr(root, 'Foreground')

    const config: BootstrapConfig = {
        version: parseInt(attr(root, 'Version') ?? '1'),
        width:   parseInt(attr(root, 'Width')   ?? '600'),
        height:  parseInt(attr(root, 'Height')  ?? '400'),
        ignoreTitleBarInset: parseBool(attr(root, 'IgnoreTitleBarInset')) ?? false,
        theme: (attr(root, 'Theme') as Theme) ?? 'Dark',
        margin: parseMargin(attr(root, 'Margin')),
        windowCornerPreference: attr(root, 'WindowCornerPreference'),
        background: rawBg && !rawBg.startsWith('{') ? rawBg : undefined,
        foreground: rawFg && !rawFg.startsWith('{') ? rawFg : undefined,
        elements: [],
    }

    const rootCtx = makeRootCtx(config.width, config.height)
    const isContainer = ROOT_CONTAINER_TAGS.has(rootTag) || rootTag.endsWith(':Window')
    config.elements = collectElements(isContainer ? root : root, rootCtx)
    config.elements.sort((a, b) => (a.zIndex ?? 0) - (b.zIndex ?? 0))

    return config
}
