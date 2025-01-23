function createInfiniteMediaStore() {
    let selectedHashes: Array<string> = $state([]);
    let onSelectMode = $derived(selectedHashes.length > 0);



    return {

        get selectedHashes() { return selectedHashes },
        get onSelectMode() { return onSelectMode },

        addMedia: (hash: string) => {

            if (selectedHashes.includes(hash as never)) {
                selectedHashes = selectedHashes.filter((h) => h !== hash);
            } else {
                selectedHashes = [...selectedHashes, hash];
            }

        },


        cleanAllMedia: () => {
            selectedHashes = [];
        }




    }
}

export const InfiniteMediaStore = createInfiniteMediaStore();