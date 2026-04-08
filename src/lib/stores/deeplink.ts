import { writable } from 'svelte/store'

const stored = localStorage.getItem('deepLinkUrl')
export const deepLinkUrl = writable<string | null>(stored)

deepLinkUrl.subscribe(value => {
    if (value) {
        localStorage.setItem('deepLinkUrl', value)
    } else {
        localStorage.removeItem('deepLinkUrl')
    }
})