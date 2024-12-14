import type { Optional } from "$lib/Option";
import type { TagQueryOutput } from "$lib/tauri_bindings";

type SearchDropDownProps = {
    entriesToShow: Array<TagQueryOutput>;
    keyboardSelectedIndex: number;
    searchContents: string;
};


type SearchTag = {
    name: string;
    count: number;
}