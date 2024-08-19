import { get } from "svelte/store";




function createMediaModalStatusStore() {

    let selectedHash = $state("");
    let isOpen = $state(false);
    let tagsEditModeActive = $state(false);

    return {
        get isOpen() { return isOpen },
        get hash() { return selectedHash },
        get tagsEditModeActive() { return tagsEditModeActive },

        open: (hash: string) => {
            if (!isOpen) {
                selectedHash = hash;
                isOpen = true;
            }

        },
        close: () => {
            isOpen = false;
            tagsEditModeActive = false;
        },

        setTagsEditModeActive(state: boolean) {
            tagsEditModeActive = state
        }
    }

}

export const MediaModalStatusStore = createMediaModalStatusStore();





