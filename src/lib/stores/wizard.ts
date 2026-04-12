import { writable, derived } from 'svelte/store'
import { goto } from '$app/navigation'

export const steps = [
    { label: 'Welcome To Crush!', description: 'A message!', path: './welcome' },
    { label: 'UIs', description: 'A tour about Uis in crush.', path: './ui' },
    { label: 'Export Configs', description: 'Export Configs from other boostraper', path: './export' },
    { label: 'Complete', description: 'Good luck', path: './complete' },
]

export const currentStep = writable(0)

export const isFirst = derived(currentStep, $s => $s === 0)
export const isLast  = derived(currentStep, $s => $s === steps.length - 1)

export function next() {
    currentStep.update(s => {
        const n = Math.min(s + 1, steps.length - 1)
        goto(steps[n].path)
        return n
    })
}

export function back() {
    currentStep.update(s => {
        const n = Math.max(s - 1, 0)
        goto(steps[n].path)
        return n
    })
}