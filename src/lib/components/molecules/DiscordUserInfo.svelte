<script lang="ts">
    import { serverService } from '$lib/stores/discordAuth.svelte'

    interface Props {
        class?: string
    }

    let { class: className = '' }: Props = $props()

    function avatarUrl(user: { id: string; avatar: string | null } | null) {
        if (!user?.avatar) return null
        return `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64`
    }
</script>

{#if serverService.isLoggedIn && serverService.user}
    <div class="flex items-center gap-2 text-sm text-stone-400 {className}">
        {#if avatarUrl(serverService.user)}
            <img
                src={avatarUrl(serverService.user)}
                alt=""
                class="w-5 h-5 rounded-full"
            />
        {/if}
        <span
            >Logged in as <b class="text-stone-200"
                >{serverService.user.username}</b
            ></span
        >
        <span class="text-stone-600">·</span>
        <span>{serverService.validatedCount} validated</span>
        <span class="text-stone-600">·</span>
        <span>{serverService.submittedCount} submissions</span>
    </div>
{:else if serverService.authError}
    <p class="text-sm text-red-400 {className}">{serverService.authError}</p>
{/if}
