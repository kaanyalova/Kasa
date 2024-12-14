function createSearchStore() {

    let searchContents = $state("");

    return {
        get searchContents() { return searchContents },
        set searchContents(value: string) {
            searchContents = value;
        }
    }
}

export const SearchStore = createSearchStore();