import Uwuifier from 'uwuifier';
import { settings } from './stores/settings.svelte';

const uwuifier = new Uwuifier({
  spaces: { faces: 0.05, actions: 0.075, stutters: 0.1 },
});

const originalText = new WeakMap<Text, string>();
const SKIP_TAGS = new Set(['SCRIPT', 'STYLE', 'TEXTAREA', 'INPUT', 'CODE', 'PRE']);

function shouldSkip(node: Node): boolean {
  const parent = node.parentElement;
  if (!parent) return true;
  return SKIP_TAGS.has(parent.tagName);
}

function applyNode(node: Text) {
  if (shouldSkip(node)) return;
  if (!originalText.has(node)) {
    if (!node.textContent?.trim()) return;
    originalText.set(node, node.textContent);
  }
  const original = originalText.get(node);
  if (original === undefined) return;
  node.textContent = settings.uwuEnabled ? uwuifier.uwuifySentence(original) : original;
}

function walkAll(root: Node) {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let node: Node | null;
  while ((node = walker.nextNode())) applyNode(node as Text);
}

let observer: MutationObserver | null = null;
const OBSERVE_OPTS: MutationObserverInit = { childList: true, subtree: true, characterData: true };


function withObserverPaused(root: HTMLElement, fn: () => void) {
  observer?.disconnect();
  fn();
  observer?.observe(root, OBSERVE_OPTS);
}

export function initUwu(root: HTMLElement = document.body) {
  walkAll(root);

  observer = new MutationObserver((mutations) => {
    withObserverPaused(root, () => {
      for (const m of mutations) {
        if (m.type === 'characterData') applyNode(m.target as Text);
        else if (m.type === 'childList') {
          m.addedNodes.forEach((n) => {
            if (n.nodeType === Node.TEXT_NODE) applyNode(n as Text);
            else if (n.nodeType === Node.ELEMENT_NODE) walkAll(n);
          });
        }
      }
    });
  });

  observer.observe(root, OBSERVE_OPTS);

  $effect.root(() => {
    $effect(() => {
      settings.uwuEnabled; // re-apply whole tree on toggle
      withObserverPaused(root, () => walkAll(root));
    });
  });
}
