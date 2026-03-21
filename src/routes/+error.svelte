<script lang="ts">
    import { invoke } from '@tauri-apps/api/core'
    import { getCurrentWindow } from '@tauri-apps/api/window'

    async function openchoicewin() {
        await invoke('create_or_focus_window', {
            label: 'crushBootstrapChoiceWindow',
            url: 'mainWin/choiceWin',
            title: 'Crush',
            width: 500.0,
            height: 250.0,
            minWidth: 500,
            minHeight: 250.0,
        })

        setTimeout(() => {
            // wait before killing to prevent crash
            getCurrentWindow().close()
        }, 100)
    }
</script>

<div
    class="flex h-screen w-full flex-col items-center justify-center gap-4 font-medium"
>
    <h1 class="text-[50px] text-center leading-tight">
        you just discovered a super super super rare error
    </h1>
    <button class="px-4 py-2 bg-[#121417]" on:click={openchoicewin}>
        go back to choice menu
    </button>
</div>
