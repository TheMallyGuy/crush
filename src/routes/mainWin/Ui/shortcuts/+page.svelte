<script lang="ts">
    import Button from '$lib/components/atoms/Button.svelte'
    import Textbox from '$lib/components/atoms/Textbox.svelte'
    import { invoke } from '@tauri-apps/api/core'
    import { appDataDir, join } from '@tauri-apps/api/path'
    import { BaseDirectory, create, exists, mkdir } from '@tauri-apps/plugin-fs'
    import { fetch } from '@tauri-apps/plugin-http'
    import { error, info } from '@tauri-apps/plugin-log'
    import { _ } from 'svelte-i18n'

    let image = $state(null as string | null)
    let gameId = $state(null as string | null)
    let status = $state($_('pages.shortcuts.creator.status.idle') as string)

    async function getGameUniverse(placeID: number): Promise<number | null> {
        const res = await fetch(
            `https://apis.roblox.com/universes/v1/places/${placeID}/universe`
        ).catch((error) => {
            throw new Error(`Network error: ${error}`)
        })

        if (res.status === 404) throw new Error('Game not found')
        if (!res.ok) throw new Error(`API error: ${res.status}`)

        const data = await res.json().catch(() => null)
        return data?.universeId ?? null
    }

    async function getThumbnailViaUniverseId(
        universeId: number
    ): Promise<string | null> {
        try {
            const res = await fetch(
                `https://thumbnails.roblox.com/v1/games/icons?universeIds=${universeId}&size=512x512&format=Png&isCircular=false`
            )

            if (!res.ok) return null

            const result = await res.json()
            return result?.data?.[0]?.imageUrl ?? null
        } catch (err) {
            error('Error fetching thumbnail:', err)
            return null
        }
    }

    async function getGameName(universeId: number) {
        try {
            const res = await fetch(
                `https://games.roblox.com/v1/games?universeIds=${universeId}`
            )

            if (!res.ok) return null

            const result = await res.json()
            console.log(result)

            return result?.data?.[0]?.name ?? null
        } catch (err) {
            error('Error fetching game name:', err)
            return null
        }
    }

    async function createShortCut() {
        const universeId = await getGameUniverse(Number(gameId))
        const gameName = await getGameName(Number(universeId))
        const imageData = await getThumbnailViaUniverseId(Number(universeId))

        if (!gameName) {
            error('Game name not found, cannot create shortcut')
            return
        }

        const imageBinary = await fetch(imageData!).then((res) =>
            res.arrayBuffer()
        )

        if (
            !(await exists('shorcutImages', { baseDir: BaseDirectory.AppData }))
        ) {
            await mkdir('shorcutImages', {
                baseDir: BaseDirectory.AppData,
            }).catch((err) => {
                error(`Error creating directory: ${err}`)
            })
        }

        const file = await create(
            `shorcutImages/${gameName}Shortcut.png`.replace(/\s/g, ''),
            {
                baseDir: BaseDirectory.AppData,
            }
        )

        await file.write(new Uint8Array(imageBinary)).catch((err) => {
            error(`Error writing thumbnail: ${err}`)
        })

        await invoke('new_shorcut', {
            name: gameName,
            gameId: Number(gameId),
            imagePath: await join(
                await appDataDir(),
                'shorcutImages',
                `${gameName}Shortcut.png`.replace(/\s/g, '')
            ),
        }).catch((err) => {
            error(`Error creating shortcut: ${err}`)
        })

        status = $_('pages.shortcuts.creator.status.done')
    }

    $effect(() => {
        if (!gameId) return
        status = $_('pages.shortcuts.creator.status.fetching')

        const timeout = setTimeout(async () => {
            try {
                const universeId = await getGameUniverse(Number(gameId))
                if (!universeId) {
                    status = $_('pages.shortcuts.creator.status.notFound')
                    image = null
                    return
                }

                const thumbnail = await getThumbnailViaUniverseId(universeId)
                image = thumbnail
                status = thumbnail
                    ? $_('pages.shortcuts.creator.status.fetched')
                    : $_('pages.shortcuts.creator.status.noThumb')
            } catch (error) {
                status = $_('pages.shortcuts.creator.status.error')
                image = null
            }
        }, 500)

        return () => clearTimeout(timeout)
    })
</script>

<div class="flex flex-col gap-8">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight text-stone-100">
                {$_('pages.shortcuts.shortcuts')}
            </h1>
            <p class="text-stone-400 mt-1">
                {$_('pages.shortcuts.description')}
            </p>
        </div>
    </div>

    <div
        class="group relative gap-10 flex w-full flex-row bg-anthracite/40 p-5 transition-all duration-150"
    >
        <div class="h-40 w-60 bg-anthracite/40 cursor-target">
            <img src={image} />
        </div>

        <div class="flex flex-col w-full gap-5">
            <Textbox
                label={$_('pages.shortcuts.creator.gameIdInput')}
                placeholder=""
                bind:value={gameId}
            />

            <div class="flex w-full flex-row gap-5 justify-between relative">
                <p class="opacity-75 text-xs text-wrap wrap-normal w-2/3">
                    {status}
                </p>

                <Button class="" on:click={createShortCut}
                    >{$_('pages.shortcuts.creator.button')}</Button
                >
            </div>
        </div>
    </div>
</div>
