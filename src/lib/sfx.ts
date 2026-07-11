const cache = new Map<string, HTMLAudioElement>();
let current: HTMLAudioElement | null = null;

export function playSfx(src: string, volume = 0.3) {
  if (current && !current.paused) {
    current.pause();
    current.currentTime = 0;
  }

  let audio = cache.get(src);
  if (!audio) {
    audio = new Audio(src);
    audio.preload = 'auto';
    cache.set(src, audio);
  }

  audio.volume = volume;
  audio.currentTime = 0;
  current = audio;
  audio.play().catch((err) => console.warn('[sfx] play() blocked:', err));
}
