import { writable } from 'svelte/store'

const stored = localStorage.getItem('launchAppType')
export const launchAppType = writable<string | null>(stored)

launchAppType.subscribe(value => {
    if (value) {
        localStorage.setItem('launchAppType', value)
    } else {
        localStorage.removeItem('launchAppType')
    }
})