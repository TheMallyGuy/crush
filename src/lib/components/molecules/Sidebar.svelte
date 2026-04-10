<script lang="ts">
    import {
        Puzzle,
        HardDrive,
        Hammer,
        Flag,
        Palette,
        Settings,
        Info,
        Menu,
    } from '@lucide/svelte'
    import { page } from '$app/stores'
    import { _ } from 'svelte-i18n';

    let baseUiUrl: string = '/mainWin/Ui'

    const mainNav = [
        {
            name: $_('pages.integrations.integrations'),
            icon: Puzzle,
            href: `${baseUiUrl}/integrations`,
        },
        {
            name: $_('pages.installations.installations'),
            icon: HardDrive,
            href: `${baseUiUrl}/installation`,
        },
        { name: $_('pages.mod.mod'), icon: Hammer, href: `${baseUiUrl}/mods/modManagement`, activePrefix: `${baseUiUrl}/mods` },
        { name: $_('pages.fastflag.fastflag'), icon: Flag, href: `${baseUiUrl}/fastflags` },
        { name: $_('pages.appearance.appearance'), icon: Palette, href: `${baseUiUrl}/appearance` },
    ]

    const footerNav = [
        { name: $_('pages.settings.settings'), icon: Settings, href: `${baseUiUrl}/settings` },
    ]
</script>

<aside
    class="flex flex-col h-screen bg-anthracite text-stone-400 border-r border-stone-800/40 p-2 pt-8 transition-all duration-150 w-16 sm:w-64"
>
    <div class="flex items-center justify-center sm:justify-start p-2 mb-4">
        <button class="hover:text-stone-100 transition-colors">
            <Menu size={20} />
        </button>
    </div>

    <nav class="flex flex-col gap-1 overflow-hidden">
        {#each mainNav as item}
           {@const isActive = $page.url.pathname.startsWith(item.activePrefix ?? item.href)}

            <a
                href={item.href}
                class="relative flex items-center justify-center sm:justify-start gap-3 px-3 py-2 rounded-lg transition-all hover:bg-stone-800/50 hover:text-stone-100 group
                {isActive ? 'bg-stone-800/80 text-sapphire' : ''}"
            >
                {#if isActive}
                    <div
                        class="absolute left-0 w-1 h-5 bg-sapphire rounded-r-full shadow-glow-sapphire"
                    ></div>
                {/if}

                <svelte:component this={item.icon} size={18} class="shrink-0" />
                <span class="hidden sm:block text-[14px] font-medium truncate"
                    >{item.name}</span
                >
            </a>
        {/each}
    </nav>

    <div class="flex-grow"></div>

    <nav
        class="flex flex-col gap-1 border-t border-stone-800/40 pt-4 overflow-hidden"
    >
        {#each footerNav as item}
            {@const isActive = $page.url.pathname === item.href}

            <a
                href={item.href}
                class="relative flex items-center justify-center sm:justify-start gap-3 px-3 py-2 rounded-lg transition-all hover:bg-stone-800/50 hover:text-stone-100 group
                {isActive ? 'bg-stone-800/80 text-sapphire' : ''}"
            >
                {#if isActive}
                    <div
                        class="absolute left-0 w-1 h-5 bg-sapphire rounded-r-full shadow-glow-sapphire"
                    ></div>
                {/if}

                <svelte:component this={item.icon} size={18} class="shrink-0" />
                <span class="hidden sm:block text-[14px] font-medium truncate"
                    >{item.name}</span
                >
            </a>
        {/each}
    </nav>
</aside>
