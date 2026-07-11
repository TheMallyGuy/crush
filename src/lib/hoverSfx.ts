import { settings } from './stores/settings.svelte';
import { playSfx } from './sfx';

const SFX_SRC = '/sfx/hover.mp3';

const INTERACTIVE_SELECTOR =
  'button:not([role="switch"]), a[href], input[type="checkbox"], input[type="radio"], input[type="file"], input[type="color"], select, [role="button"], [role="tab"], [role="menuitem"], [role="checkbox"]';

let lastTarget: Element | null = null;

function handlePointerOver(event: PointerEvent) {
  if (!settings.hoverSfxEnabled) return;
  const target = (event.target as Element | null)?.closest(INTERACTIVE_SELECTOR) ?? null;
  if (!target || target === lastTarget) return;
  lastTarget = target;
  playSfx(SFX_SRC);
}

function handlePointerOut(event: PointerEvent) {
  const related = event.relatedTarget as Element | null;
  if (lastTarget && (!related || !lastTarget.contains(related))) lastTarget = null;
}

function handleClick(event: MouseEvent) {
  if (!settings.hoverSfxEnabled) return;
  const target = (event.target as Element | null)?.closest(INTERACTIVE_SELECTOR) ?? null;
  if (!target) return;
  playSfx(SFX_SRC);
}

export function initHoverSfx(root: HTMLElement = document.body) {
  root.addEventListener('pointerover', handlePointerOver);
  root.addEventListener('pointerout', handlePointerOut);
  root.addEventListener('click', handleClick);
}
