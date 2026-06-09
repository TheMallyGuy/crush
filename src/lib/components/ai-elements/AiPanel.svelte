<script lang="ts">
    import { listen } from '@tauri-apps/api/event'
    import { invoke } from '@tauri-apps/api/core'
    import { fly } from 'svelte/transition'
    import { onMount, onDestroy } from 'svelte'
    import { info } from '@tauri-apps/plugin-log'

    type State = 'idle' | 'listening' | 'thinking' | 'responding' | 'done'
    type Message = { role: 'user' | 'assistant'; text: string }

    let phase = $state<State>('idle')
    let visible = $derived(phase !== 'idle')

    let history = $state<Message[]>([])   // completed exchanges
    let currentQ = $state('')             // question in progress
    let currentA = $state('')             // answer being streamed

    let cards: { label: string; value: string }[] = $state([])
    let interim = $state('')
    let question = $state('')
    let modelLoading = $state(false)
    let bodyEl: HTMLDivElement | undefined = $state()
    let unlisteners: (() => void)[] = []
    let speechBuffer = ''
    let sendTimer: ReturnType<typeof setTimeout> | null = null
    let dismissTimer: ReturnType<typeof setTimeout> | null = null
    const SEND_DELAY_MS = 80
    const AUTO_DISMISS_MS = 30_000

    let asr: any = null
    let audioCtx: AudioContext | null = null
    let mediaStream: MediaStream | null = null
    let workletNode: AudioWorkletNode | null = null
    let audioChunks: Float32Array[] = []
    let isSpeaking = false
    let silenceSamples = 0
    let speechSamples = 0
    const SAMPLE_RATE = 16000
    const CHUNK_SIZE = 128
    const SILENCE_SAMPLES = Math.round(0.4 * SAMPLE_RATE)
    const RMS_THRESHOLD = 0.03
    const MIN_SPEECH_SAMPLES = Math.round(0.25 * SAMPLE_RATE)
    const MIN_TEXT_LENGTH = 3

    const SHAPE: Record<State, { w: number; h: number; r: number }> = {
        idle:       { w: 0,   h: 0,   r: 0  },
        listening:  { w: 230, h: 40,  r: 20 },
        thinking:   { w: 44,  h: 44,  r: 22 },
        responding: { w: 360, h: 280, r: 14 },
        done:       { w: 360, h: 280, r: 14 },
    }

    let shape = $derived(SHAPE[phase])

    $effect(() => {
        if (dismissTimer) { clearTimeout(dismissTimer); dismissTimer = null }
        if (phase === 'done') {
            dismissTimer = setTimeout(() => { dismissTimer = null; dismiss() }, AUTO_DISMISS_MS)
        }
    })

    $effect(() => {
        const s = SHAPE[phase]
        const pad = 16
        const w = s.w > 0 ? s.w + pad * 2 : 0
        const h = s.h > 0 ? s.h + pad * 2 : 0
        invoke('resize_overlay', { w, h })
    })

    $effect(() => {
        // track currentA and history length to re-run
        const _ = currentA + history.length
        if (bodyEl) bodyEl.scrollTop = bodyEl.scrollHeight
    })

    let transition = $derived(
        phase === 'thinking'
            ? 'width .28s ease-in, height .28s ease-in, border-radius .28s ease-in'
            : 'width .42s cubic-bezier(0.34,1.56,0.64,1), height .42s cubic-bezier(0.34,1.56,0.64,1), border-radius .32s ease'
    )

    function archiveExchange() {
        if (currentQ && currentA.trim()) {
            history = [
                ...history,
                { role: 'user', text: currentQ },
                { role: 'assistant', text: currentA },
            ]
        }
    }

    function dismiss() {
        stopMic()
        phase = 'idle'
        history = []
        currentQ = ''
        currentA = ''
        interim = ''
        question = ''
        speechBuffer = ''
        if (sendTimer) { clearTimeout(sendTimer); sendTimer = null }
        if (dismissTimer) { clearTimeout(dismissTimer); dismissTimer = null }
        modelLoading = false
        cards = []
        invoke('clear_ai_history')
    }

    function stopMic() {
        if (workletNode) { try { workletNode.disconnect() } catch {} workletNode = null }
        if (mediaStream) { mediaStream.getTracks().forEach((t) => t.stop()); mediaStream = null }
        if (audioCtx) { try { audioCtx.close() } catch {}; audioCtx = null }
        audioChunks = []; isSpeaking = false; silenceSamples = 0; speechSamples = 0
    }

    function mergeChunks(chunks: Float32Array[]): Float32Array {
        const total = chunks.reduce((n, c) => n + c.length, 0)
        const merged = new Float32Array(total)
        let offset = 0
        for (const c of chunks) { merged.set(c, offset); offset += c.length }
        return merged
    }

    async function commitAudio(chunks: Float32Array[]) {
        if (chunks.length === 0) return
        const audio = mergeChunks(chunks)
        try {
            const result: any = await asr(audio, { sampling_rate: SAMPLE_RATE })
            const text = (result?.text ?? '').trim()
            if (!text || text.length < MIN_TEXT_LENGTH) return
            speechBuffer += (speechBuffer ? ' ' : '') + text
            currentQ = speechBuffer
            interim = ''

            if (sendTimer) clearTimeout(sendTimer)
            sendTimer = setTimeout(() => {
                sendTimer = null
                const prompt = speechBuffer
                speechBuffer = ''
                if (!prompt) return
                archiveExchange()
                currentQ = prompt
                currentA = ''
                invoke('ask_ai', { prompt })
            }, SEND_DELAY_MS)
        } catch (err) {
            console.warn('[Whisper] transcription error:', err)
        }
    }

    export async function startListening() {
        phase = 'listening'
        interim = ''
        modelLoading = false

        if (!asr) {
            modelLoading = true
            try {
                const { pipeline, env } = await import('@xenova/transformers')
                env.allowLocalModels = false
                asr = await pipeline('automatic-speech-recognition', 'Xenova/whisper-tiny.en')
            } catch (err) {
                console.error('[Whisper] model load failed:', err)
                modelLoading = false
                return
            }
            modelLoading = false
        }

        try {
            mediaStream = await navigator.mediaDevices.getUserMedia({
                audio: { channelCount: 1, sampleRate: SAMPLE_RATE, echoCancellation: true, noiseSuppression: true },
            })
        } catch {
            console.warn('[Whisper] mic access denied')
            return
        }

        audioCtx = new AudioContext({ sampleRate: SAMPLE_RATE })
        const source = audioCtx.createMediaStreamSource(mediaStream)

        await audioCtx.audioWorklet.addModule('/audio-worklet.js')
        workletNode = new AudioWorkletNode(audioCtx, 'speech-processor')

        workletNode.port.onmessage = (e: MessageEvent<Float32Array>) => {
            if (phase !== 'listening') return
            const input = e.data
            const rms = Math.sqrt(input.reduce((s, v) => s + v * v, 0) / input.length)

            if (rms > RMS_THRESHOLD) {
                isSpeaking = true; silenceSamples = 0; speechSamples += CHUNK_SIZE
                audioChunks.push(input); interim = '…'
            } else if (isSpeaking) {
                silenceSamples += CHUNK_SIZE; audioChunks.push(input)
                if (silenceSamples >= SILENCE_SAMPLES) {
                    const committed = audioChunks
                    const hadEnoughSpeech = speechSamples >= MIN_SPEECH_SAMPLES
                    audioChunks = []; isSpeaking = false; silenceSamples = 0; speechSamples = 0
                    if (hadEnoughSpeech) commitAudio(committed)
                }
            }
        }

        source.connect(workletNode)
        workletNode.connect(audioCtx.destination)
    }

    onMount(async () => {
        unlisteners = await Promise.all([
            listen('ai:thinking', () => {
                stopMic()
                cards = []
                phase = 'thinking'
            }),
            listen<string>('ai:token', (e) => {
                if (phase !== 'responding') phase = 'responding'
                currentA += e.payload
            }),
            listen('ai:done', () => {
                phase = 'done'
            }),
            listen<{ label: string; value: string }[]>('ai:cards', (e) => {
                cards = e.payload
            }),
            listen('ai:listen', () => startListening()),
        ])
    })

    onDestroy(() => {
        unlisteners.forEach((u) => u())
        stopMic()
    })
</script>

{#if visible}
    <div
        class="fixed top-3 left-1/2 -translate-x-1/2 overflow-hidden bg-obsidian/95 backdrop-blur-2xl border border-white/[0.08] z-50 cursor-default"
        style="width:{shape.w}px; height:{shape.h}px; border-radius:{shape.r}px; transition:{transition};"
        transition:fly={{ y: -56, duration: 280, opacity: 0 }}
        role="complementary"
        aria-live="polite"
    >
        {#if phase === 'listening'}
            <!-- PILL -->
            <div class="flex items-center gap-2.5 w-full h-full px-3 pl-3.5 box-border">
                {#if modelLoading}
                    <span class="w-3.5 h-3.5 shrink-0 rounded-full border border-sapphire/25 border-t-sapphire animate-[ai-spin_0.8s_linear_infinite]"></span>
                    <span class="flex-1 text-[11px] font-medium text-stone-600 tracking-[0.02em]">Loading model…</span>
                {:else}
                    <div class="flex items-center gap-[2.5px] shrink-0" aria-hidden="true">
                        {#each [0.35, 0.7, 1, 0.7, 0.35] as base, i}
                            <div
                                class="w-[3px] bg-sapphire origin-center animate-[ai-dance_0.5s_ease-in-out_infinite_alternate]"
                                style="height:{base * 20}px; animation-delay:{i * 0.09}s;"
                            ></div>
                        {/each}
                    </div>
                    <span class="flex-1 text-[11px] font-medium text-stone-400 truncate tracking-[0.02em]">
                        {interim || currentQ || 'Listening…'}
                    </span>
                {/if}
                <button
                    class="grid place-items-center w-[22px] h-[22px] shrink-0 bg-transparent border-none cursor-pointer text-stone-600 hover:text-stone-100 transition-colors duration-150 p-0"
                    onclick={dismiss}
                    aria-label="Cancel"
                >
                    <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
                        <path d="M1 1l6 6M7 1L1 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                    </svg>
                </button>
            </div>

        {:else if phase === 'thinking'}
            <!-- CIRCLE -->
            <div class="w-full h-full grid place-items-center">
                <span class="block w-[22px] h-[22px] rounded-full border-2 border-sapphire/20 border-t-sapphire animate-[ai-spin_0.8s_linear_infinite]"></span>
            </div>

        {:else}
            <!-- CARD -->
            <div class="flex flex-col w-full h-full overflow-hidden">
                <div
                    class="flex-1 overflow-y-auto p-2.5 px-3 flex flex-col gap-1 scrollbar-hide"
                    bind:this={bodyEl}
                >
                    <!-- Past exchanges -->
                    {#each history as msg, i}
                        {#if msg.role === 'user'}
                            {#if i > 0}
                                <div class="h-px bg-white/[0.04] my-0.5"></div>
                            {/if}
                            <p class="text-[11px] text-stone-500 m-0 py-1.5 px-2.5 bg-white/[0.03] border border-white/5 text-right italic self-end max-w-[85%]">
                                {msg.text}
                            </p>
                        {:else}
                            <p class="text-xs leading-relaxed text-stone-400 m-0 py-2 px-2.5">
                                {msg.text}
                            </p>
                        {/if}
                    {/each}

                    <!-- Current exchange -->
                    {#if currentQ}
                        {#if history.length > 0}
                            <div class="h-px bg-white/[0.04] my-0.5"></div>
                        {/if}
                        <p class="text-[11px] text-stone-500 m-0 py-1.5 px-2.5 bg-white/[0.03] border border-white/5 text-right italic self-end max-w-[85%]">
                            {currentQ}
                        </p>
                    {/if}
                    {#if currentA}
                        <p class="text-xs leading-relaxed text-stone-200 m-0 bg-sapphire/5 border border-sapphire/10 py-2 px-2.5">
                            {currentA}{#if phase === 'responding'}<span
                                class="inline-block w-[2px] h-[11px] bg-sapphire align-middle ml-[2px] animate-[ai-blink_1s_step-end_infinite]"
                            ></span>{/if}
                        </p>
                    {/if}

                    {#each cards as card (card.label)}
                        <div class="bg-white/[0.02] border border-white/5 py-2 px-2.5 hover:border-white/10 transition-colors duration-150">
                            <span class="block text-[10px] font-medium uppercase tracking-[0.08em] text-stone-600 mb-1">{card.label}</span>
                            <p class="text-xs text-stone-400 m-0 leading-relaxed">{card.value}</p>
                        </div>
                    {/each}
                </div>

                <!-- Dismiss button -->
                <div class="flex justify-end px-3 pb-2 pt-1 shrink-0">
                    <button
                        class="text-[10px] text-stone-700 hover:text-stone-400 transition-colors duration-150 bg-transparent border-none cursor-pointer p-0"
                        onclick={dismiss}
                    >dismiss</button>
                </div>
            </div>
        {/if}
    </div>
{/if}
