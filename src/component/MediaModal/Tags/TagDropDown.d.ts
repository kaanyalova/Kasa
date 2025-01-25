import type { TagQueryOutput } from "$lib/tauri_bindings"

type TagDropDownProps = {
    top: number,
    left: number,
    selectedIndex: number,
    tags: Array<TagQueryOutput>
    onTagClick: (index: number) => void
}