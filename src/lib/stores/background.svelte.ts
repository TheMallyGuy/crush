import { load } from '@tauri-apps/plugin-store'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile, writeFile, mkdir, exists } from '@tauri-apps/plugin-fs'
import { appDataDir, join, basename } from '@tauri-apps/api/path'
import { convertFileSrc } from '@tauri-apps/api/core'

export type BackgroundType = 'default' | 'color' | 'image'

interface BackgroundData {
    type: BackgroundType
    color: string
    image: string // stored filename inside <appData>/backgrounds
}

const DEFAULT_COLOR = '#0a0a0a'

const DEFAULTS: BackgroundData = {
    type: 'default',
    color: DEFAULT_COLOR,
    image: '',
}

function safeName(name: string): string {
    return name.replace(/[^a-zA-Z0-9._-]/g, '_').slice(0, 64)
}

class BackgroundSettings {
    type = $state<BackgroundType>('default')
    color = $state(DEFAULT_COLOR)
    image = $state('') // stored filename
    imageUrl = $state('') // resolved convertFileSrc url for display/use
    loaded = $state(false)

    async init() {
        const store = await load('config.json')
        const saved = await store.get<BackgroundData>('background')
        const data = { ...DEFAULTS, ...saved }

        this.type = data.type
        this.color = data.color || DEFAULT_COLOR
        this.image = data.image || ''

        if (this.image) {
            try {
                const appData = await appDataDir()
                const path = await join(appData, 'backgrounds', this.image)
                if (await exists(path)) {
                    this.imageUrl = convertFileSrc(path)
                } else {
                    this.image = ''
                    if (this.type === 'image') this.type = 'default'
                }
            } catch {
                this.image = ''
                if (this.type === 'image') this.type = 'default'
            }
        }

        this.loaded = true
        this.apply()
    }

    apply() {
        if (typeof document === 'undefined') return
        const body = document.body
        if (!body) return

        if (this.type === 'image' && this.imageUrl) {
            body.style.background = `${DEFAULT_COLOR} url("${this.imageUrl}") center / cover no-repeat fixed`
        } else if (this.type === 'color') {
            body.style.background = this.color || DEFAULT_COLOR
        } else {
            body.style.background = DEFAULT_COLOR
        }
    }

    async save() {
        const store = await load('config.json')
        await store.set('background', {
            type: this.type,
            color: this.color,
            image: this.image,
        } satisfies BackgroundData)
        await store.save()
        this.apply()
    }

    async setType(type: BackgroundType) {
        this.type = type
        await this.save()
    }

    async setColor(color: string) {
        this.color = color
        this.type = 'color'
        await this.save()
    }

    async pickImage(): Promise<boolean> {
        const selected = await open({
            title: 'Select Background Image',
            filters: [
                {
                    name: 'Images',
                    extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp'],
                },
            ],
            multiple: false,
        })

        if (!selected || typeof selected !== 'string') return false

        const appData = await appDataDir()
        const bgDir = await join(appData, 'backgrounds')
        if (!(await exists(bgDir))) {
            await mkdir(bgDir, { recursive: true })
        }

        const rawBase = await basename(selected)
        const filename = `${Date.now()}_${safeName(rawBase)}`
        const destPath = await join(bgDir, filename)

        const bytes = await readFile(selected)
        await writeFile(destPath, bytes)

        this.image = filename
        this.imageUrl = convertFileSrc(destPath)
        this.type = 'image'
        await this.save()
        return true
    }

    async reset() {
        this.type = 'default'
        await this.save()
    }
}

export const background = new BackgroundSettings()
