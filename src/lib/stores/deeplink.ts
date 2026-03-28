import { writable } from "svelte/store";

export const deepLinkUrl = writable<string | null>(null);