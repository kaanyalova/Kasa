import type { Optional } from "$lib/Option";
import type { TagQueryOutput } from "$lib/tauri_bindings";

type SearchDropDownProps = {
    entriesToShow: Array<TagQueryOutput>;
    keyboardSelectedIndex: number;
    searchContents: String;
};


type SearchTag = {
    name: string;
    count: number;
}