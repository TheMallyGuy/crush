<script>
  import { 
    Puzzle, HardDrive, Hammer, Flag, Palette, 
    Settings, Info, Menu
  } from '@lucide/svelte';
  import { page } from '$app/stores'; 

  const mainNav = [
    { name: 'Integrations', icon: Puzzle, href: '/integrations' },
    { name: 'Installation', icon: HardDrive, href: '/installation' },
    { name: 'Mods', icon: Hammer, href: '/mods' },
    { name: 'FastFlag', icon: Flag, href: '/fastflags' },
    { name: 'Appearance', icon: Palette, href: '/appearance' },
  ];

  const footerNav = [
    { name: 'Settings', icon: Settings, href: '/settings' },
    { name: 'About', icon: Info, href: '/about' },
  ];
</script>

<aside class="flex flex-col h-screen bg-[#121417] text-[#a0a0a0] border-r border-[#222] p-2 transition-all duration-300 w-16 sm:w-64">
  
  <div class="flex items-center justify-center sm:justify-start p-2 mb-4">
    <button class="hover:text-white transition-colors">
      <Menu size={20} />
    </button>
  </div>

  <nav class="flex flex-col gap-1 overflow-hidden">
    {#each mainNav as item}
      {@const isActive = $page.url.pathname.startsWith(item.href)}
      
      <a href={item.href} 
         class="relative flex items-center justify-center sm:justify-start gap-3 px-3 py-2 rounded-md transition-all hover:bg-[#2a2d32] hover:text-white group
                {isActive ? 'bg-[#2a2d32] text-white' : ''}">
        
        {#if isActive}
          <div class="absolute left-0 w-1 h-5 bg-blue-500 rounded-r-full"></div>
        {/if}

        <svelte:component this={item.icon} size={18} class="shrink-0" />
        <span class="hidden sm:block text-[14px] truncate">{item.name}</span>
      </a>
    {/each}
  </nav>

  <div class="flex-grow"></div>

  <nav class="flex flex-col gap-1 border-t border-[#222] pt-4 overflow-hidden">
    {#each footerNav as item}
      {@const isActive = $page.url.pathname === item.href}
      
      <a href={item.href} 
         class="relative flex items-center justify-center sm:justify-start gap-3 px-3 py-2 rounded-md transition-all hover:bg-[#2a2d32] hover:text-white
                {isActive ? 'bg-[#2a2d32] text-white' : ''}">
        
        {#if isActive}
          <div class="absolute left-0 w-1 h-5 bg-blue-500 rounded-r-full"></div>
        {/if}

        <svelte:component this={item.icon} size={18} class="shrink-0" />
        <span class="hidden sm:block text-[14px] truncate">{item.name}</span>
      </a>
    {/each}
  </nav>
</aside>