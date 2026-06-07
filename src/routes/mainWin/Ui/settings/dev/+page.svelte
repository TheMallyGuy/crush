<script lang="ts">
    import {
        getMostPlayedGame,
        getPlayedGames,
        getPlayedSessions,
        personallyDeterminePlayerType,
        PLAYER_CATEGORIES_2026,
    } from '$lib/robloxWarpped'
    import { onMount } from 'svelte'

    onMount(async () => {
        const games = getPlayedGames()
        console.log(games)

        const sessions = await getPlayedSessions()
        console.log(sessions)

        const mostPlayedGame = getMostPlayedGame(sessions)
        console.log(mostPlayedGame)

        const playerType = personallyDeterminePlayerType(sessions, PLAYER_CATEGORIES_2026)
        console.log(playerType)
    })
</script>
