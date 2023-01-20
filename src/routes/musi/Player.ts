import { writable } from "svelte/store"

export const playing = writable("")
export const progress = writable(0)
export const paused = writable(true)
