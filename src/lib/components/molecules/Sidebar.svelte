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

    let baseUiUrl: string = '/mainWin/Ui'

    const mainNav = [
        {
            name: 'Integrations',
            icon: Puzzle,
            href: `${baseUiUrl}/integrations`,
        },
        {
            name: 'Installation',
            icon: HardDrive,
            href: `${baseUiUrl}/installation`,
        },
        { name: 'Mods', icon: Hammer, href: `${baseUiUrl}/mods` },
        { name: 'FastFlag', icon: Flag, href: `${baseUiUrl}/fastflags` },
        { name: 'Appearance', icon: Palette, href: `${baseUiUrl}/appearance` },
    ]

    const footerNav = [
        { name: 'Settings', icon: Settings, href: `${baseUiUrl}/settings` },
        { name: 'About', icon: Info, href: `${baseUiUrl}/about` },
    ]
</script>

<aside
    class="flex flex-col h-screen bg-anthracite/80 backdrop-blur-md text-stone-400 border-r border-white/5 p-2 transition-all duration-300 w-16 sm:w-64 z-40"
>
    <div class="flex items-center justify-center sm:justify-start p-3 mb-6">
        <div class="flex items-center gap-3">
             <div class="w-8 h-8 rounded-lg bg-sapphire/10 flex items-center justify-center border border-sapphire/20 shadow-sapphire">
                <div class="w-4 h-4 rounded-sm bg-sapphire animate-pulse"></div>
             </div>
             <span class="hidden sm:block font-bold text-white tracking-wider text-sm uppercase">Crush</span>
        </div>
    </div>

    <nav class="flex flex-col gap-1.5 overflow-hidden">
        {#each mainNav as item}
            {@const isActive = $page.url.pathname.startsWith(item.href)}

            <a
                href={item.href}
                class="relative flex items-center justify-center sm:justify-start gap-3 px-3 py-2.5 rounded-item transition-all hover:bg-white/5 hover:text-white group
                {isActive ? 'bg-sapphire/10 text-white shadow-sapphire' : ''}"
            >
                {#if isActive}
                    <div
                        class="absolute left-0 w-1 h-5 bg-sapphire rounded-r-full shadow-[0_0_8px_rgba(59,130,246,0.8)]"
                    ></div>
                {/if}

                <svelte:component this={item.icon} size={18} class="shrink-0 {isActive ? 'text-sapphire' : 'group-hover:text-sapphire/80 transition-colors'}" />
                <span class="hidden sm:block text-[14px] font-medium truncate"
                    >{item.name}</span
                >
            </a>
        {/each}
    </nav>

    <div class="flex-grow"></div>

    <nav
        class="flex flex-col gap-1.5 border-t border-white/5 pt-4 overflow-hidden"
    >
        {#each footerNav as item}
            {@const isActive = $page.url.pathname === item.href}

            <a
                href={item.href}
                class="relative flex items-center justify-center sm:justify-start gap-3 px-3 py-2.5 rounded-item transition-all hover:bg-white/5 hover:text-white group
                {isActive ? 'bg-sapphire/10 text-white shadow-sapphire' : ''}"
            >
                {#if isActive}
                    <div
                        class="absolute left-0 w-1 h-5 bg-sapphire rounded-r-full shadow-[0_0_8px_rgba(59,130,246,0.8)]"
                    ></div>
                {/if}

                <svelte:component this={item.icon} size={18} class="shrink-0 {isActive ? 'text-sapphire' : 'group-hover:text-sapphire/80 transition-colors'}" />
                <span class="hidden sm:block text-[14px] font-medium truncate"
                    >{item.name}</span
                >
            </a>
        {/each}
    </nav>
</aside>
