<script lang="ts">
    import {
        Puzzle,
        HardDrive,
        Hammer,
        Flag,
        Palette,
        Settings,
        SlidersHorizontal,
        Grid2x2Plus,
        UsersRound,
    } from '@lucide/svelte'
    import { page } from '$app/stores'
    import { _ } from 'svelte-i18n'
    import * as Sidebar from '$lib/components/ui/sidebar/index.js'
    import { fly } from 'svelte/transition'

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
        {
            name: $_('pages.mod.mod'),
            icon: Hammer,
            href: `${baseUiUrl}/mods/modManagement`,
            activePrefix: `${baseUiUrl}/mods`,
        },
        {
            name: $_('pages.fastflag.fastflag'),
            icon: Flag,
            href: `${baseUiUrl}/fastflags`,
        },
        {
            name: $_('pages.appearance.appearance'),
            icon: Palette,
            href: `${baseUiUrl}/appearance`,
        },
        {
            name: $_('pages.gbs.gbs'),
            icon: SlidersHorizontal,
            href: `${baseUiUrl}/gbs`,
        },
        {
            name: $_('pages.shortcuts.shortcuts'),
            icon: Grid2x2Plus,
            href: `${baseUiUrl}/shortcuts`,
        },
        {
            name: $_('pages.accountManagement.accountManagement'),
            icon: UsersRound,
            href: `${baseUiUrl}/accountManagement`,
        },
    ]

    const footerNav = [
        {
            name: $_('pages.settings.settings'),
            icon: Settings,
            href: `${baseUiUrl}/settings`,
        },
    ]
</script>

<Sidebar.Root
    class="backdrop-blur-2xl  border-r border-sidebar-border text-sidebar-foreground select-none"
>
    <Sidebar.Content class="p-2 pt-4 flex flex-col h-full overflow-hidden">
        <Sidebar.Group class="p-0">
            <Sidebar.GroupContent>
                <Sidebar.Menu class="gap-1.5">
                    {#each mainNav as item, i (item.name)}
                        {@const isActive = $page.url.pathname.startsWith(
                            item.activePrefix ?? item.href
                        )}
                        <Sidebar.MenuItem>
                            <div
                                in:fly={{ y: 4, duration: 200, delay: i * 30 }}
                            >
                                <Sidebar.MenuButton
                                    {isActive}
                                    class="relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 ease-out group h-10
                                           {isActive
                                       ? 'bg-sidebar-accent text-sapphire shadow-[inset_0_1px_0_0_rgba(255,255,255,0.05)]'
                                       : 'hover:bg-sidebar-accent/50 hover:text-sidebar-foreground hover:translate-x-0.5'}"
                                >
                                    {#snippet child({ props })}
                                        <a href={item.href} {...props}>
                                            {#if isActive}
                                                <div
                                                    class="absolute left-0 w-1 h-5 bg-sapphire rounded-r-full shadow-glow-sapphire origin-left transition-all duration-300 scale-y-100"
                                                ></div>
                                            {/if}

                                            <svelte:component
                                                this={item.icon}
                                                size={18}
                                            />

                                            <span
                                                class="text-[14px] font-medium truncate transition-colors duration-200"
                                                >{item.name}</span
                                            >
                                        </a>
                                    {/snippet}
                                </Sidebar.MenuButton>
                            </div>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>

        <div class="flex-grow"></div>

        <Sidebar.SidebarFooter class="p-0 pt-4 border-t border-sidebar-border">
            <Sidebar.Menu class="gap-1.5">
                {#each footerNav as item (item.name)}
                    {@const isActive = $page.url.pathname === item.href}
                    <Sidebar.MenuItem>
                        <Sidebar.MenuButton
                            {isActive}
                            class="relative flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 ease-out group h-10
                            {isActive
                                ? 'bg-sidebar-accent text-sapphire'
                                : 'hover:bg-sidebar-accent/50 hover:text-sidebar-foreground hover:translate-x-0.5'}"
                        >
                            {#snippet child({ props })}
                                <a href={item.href} {...props}>
                                    {#if isActive}
                                        <div
                                            class="absolute left-0 w-1 h-5 bg-sapphire rounded-r-full shadow-glow-sapphire"
                                        ></div>
                                    {/if}

                                    <div
                                        class="transition-transform duration-200 group-hover:rotate-45 shrink-0"
                                    >
                                        <svelte:component
                                            this={item.icon}
                                            size={18}
                                        />
                                    </div>

                                    <span
                                        class="text-[14px] font-medium truncate"
                                        >{item.name}</span
                                    >
                                </a>
                            {/snippet}
                        </Sidebar.MenuButton>
                    </Sidebar.MenuItem>
                {/each}
            </Sidebar.Menu>
        </Sidebar.SidebarFooter>
    </Sidebar.Content>
</Sidebar.Root>
