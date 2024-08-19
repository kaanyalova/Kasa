import type { Optional } from "$lib/Option";

type SearchDropDownProps = {
    entriesToShow: Array<SearchTag>;
    keyboardSelectedIndex: number;
    searchContents: String;
};


type SearchTag = {
    name: string;
    count: number;
}