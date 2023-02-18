import { onMounted, onUnmounted } from "vue";

/** Attaches a global event listener on mount, and detaches it on unmount */
export function useListener<K extends keyof WindowEventMap>(event: K, cb: (this: Window, ev: WindowEventMap[K]) => any) {
    onMounted(() => {
        addEventListener(event, cb)
    })

    onUnmounted(() => {
        removeEventListener(event, cb)
    })
}