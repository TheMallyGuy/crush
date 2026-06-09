<div class="flex flex-col gap-1.5 w-full">
    <div class="flex items-center gap-1.5 mb-1">
        <span class="w-[5px] h-[5px] rounded-full bg-sapphire shrink-0 animate-[ai-pulse_1.2s_ease-in-out_infinite]"></span>
        <span class="text-[10px] font-medium uppercase tracking-[0.08em] text-stone-600">Thinking</span>
    </div>
    <div class="skel" style="width:72%"></div>
    <div class="skel" style="width:100%; animation-delay:0.08s"></div>
    <div class="skel" style="width:85%; animation-delay:0.16s"></div>
    <div class="skel" style="width:55%; animation-delay:0.24s"></div>
</div>

<style>
    @keyframes ai-pulse {
        0%, 100% { opacity: 1; }
        50%       { opacity: 0.25; }
    }
    @keyframes ai-sweep {
        0%   { background-position: 200% center; }
        100% { background-position: -200% center; }
    }

    .skel {
        height: 9px;
        background: linear-gradient(
            90deg,
            rgba(255,255,255,0.04) 25%,
            rgba(59,130,246,0.14) 50%,
            rgba(255,255,255,0.04) 75%
        );
        background-size: 200% 100%;
        animation: ai-sweep 1.6s ease-in-out infinite;
    }
</style>
