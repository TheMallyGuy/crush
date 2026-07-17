import { writable } from 'svelte/store'

const stored = localStorage.getItem('operating_system') ?? undefined
export const operating_system = writable<string | undefined>(stored)

operating_system.subscribe(value => {
    if (value) {
        localStorage.setItem('operating_system', value)
    } else {
        localStorage.removeItem('operating_system')
    }
})