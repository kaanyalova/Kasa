import { info } from "@tauri-apps/plugin-log";
import { onMount } from "svelte";
import { writable } from "svelte/store";


// TODO hide sidebar if size is zero? 

function updateSidebarSize() {
    const root: any = document.querySelector(':root');
    root.style.setProperty('--main-val', sidebarStore.size + 'px');
}


function createSidebarStore() {

    let isActive = $state(true);
    let sidebarSize = $state(100);
    let savedSidebarSize = $state(100);


    return {
        toggle: () => {
            // close
            if (isActive) {
                savedSidebarSize = sidebarSize;
                sidebarSize = 0;
                updateSidebarSize()
                isActive = false;
            }

            // open
            else if (!isActive) {
                sidebarSize = savedSidebarSize;
                updateSidebarSize()
                isActive = true;
            }
        },


        setSize: (size: number) => {
            sidebarSize = size;
            updateSidebarSize();
        },
        get isActive() { return isActive },
        get size() { return sidebarSize }
    }
}

export const sidebarStore = createSidebarStore();
