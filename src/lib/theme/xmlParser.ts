import type {
    Theme,
    HAlign,
    VAlign,
    Visibility,
    Margin,
    BootstrapElement,
    BootstrapConfig,
} from '$lib/types'

const attr = (el: Element, name: string): string | undefined =>
    el.getAttribute(name) ?? undefined

function parseMargin(raw: string | undefined): Margin | undefined {
    if (!raw) return undefined
    const p = raw.split(',').map(Number)
    if (p.length === 4)
        return { left: p[0], top: p[1], right: p[2], bottom: p[3] }
    if (p.length === 1)
        return { left: p[0], top: p[0], right: p[0], bottom: p[0] }
    return undefined
}

const parseNum = (v?: string): number | undefined =>
    v === undefined || v === 'Auto' ? undefined : isNaN(+v) ? undefined : +v

const parseBool = (v?: string): boolean | undefined =>
    v === undefined ? undefined : v === 'True'

export function parseXml(xml: string): BootstrapConfig {
    const doc = new window.DOMParser().parseFromString(xml, 'text/xml')
    const root = doc.documentElement

    const parseErr = root.querySelector('parsererror')
    if (parseErr) {
        throw new Error(`XML parse error: ${parseErr.textContent?.trim()}`)
    }

    const config: BootstrapConfig = {
        version: parseInt(attr(root, 'Version') ?? '1'),
        height: parseInt(attr(root, 'Height') ?? '400'),
        width: parseInt(attr(root, 'Width') ?? '600'),
        ignoreTitleBarInset:
            parseBool(attr(root, 'IgnoreTitleBarInset')) ?? false,
        theme: (attr(root, 'Theme') as Theme) ?? 'Dark',
        margin: parseMargin(attr(root, 'Margin')),
        windowCornerPreference: attr(root, 'WindowCornerPreference'),
        elements: [],
    }

    function findImageSources(node: Element): string[] {
        const sources: string[] = []
        const attributes = ['Source', 'ImageSource', 'Background', 'Foreground']

        for (const a of attributes) {
            const val = attr(node, a)
            if (
                val &&
                (val.startsWith('theme://') ||
                    (!val.includes('://') && val.endsWith('.png')) ||
                    val.endsWith('.jpg') ||
                    val.endsWith('.gif'))
            ) {
                sources.push(val)
            }
        }

        for (let i = 0; i < node.childNodes.length; i++) {
            const child = node.childNodes[i]
            if (child.nodeType === 1) {
                sources.push(...findImageSources(child as Element))
            }
        }
        return sources
    }

    for (let i = 0; i < root.childNodes.length; i++) {
        const node = root.childNodes[i] as Element
        if (node.nodeType !== 1) continue

        const sources = findImageSources(node)

        const el: BootstrapElement = {
            type: node.tagName,
            name: attr(node, 'Name'),
            hAlign: (attr(node, 'HorizontalAlignment') ||
                attr(node, 'HAlign')) as HAlign,
            vAlign: (attr(node, 'VerticalAlignment') ||
                attr(node, 'VAlign')) as VAlign,
            margin: parseMargin(attr(node, 'Margin')),
            opacity: parseNum(attr(node, 'Opacity')),
            zIndex:
                parseNum(attr(node, 'Panel.ZIndex')) ??
                parseNum(attr(node, 'ZIndex')),
            visibility: attr(node, 'Visibility') as Visibility,
            width: parseNum(attr(node, 'Width')),
            height: parseNum(attr(node, 'Height')),
            props: {},
        }

        for (let j = 0; j < node.attributes.length; j++) {
            const a = node.attributes[j]
            el.props[a.name] = a.value
        }

        if (sources.length > 0) {
            el.props['source'] = sources[0]
        }

        config.elements.push(el)
    }

    config.elements.sort((a, b) => (a.zIndex ?? 0) - (b.zIndex ?? 0))
    return config
}
