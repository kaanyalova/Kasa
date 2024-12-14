import { info } from "@tauri-apps/plugin-log";


export function handleSelect(selection: string, searchContents: string): string {
    const queries = searchContents.split(',');
    queries[queries.length - 1] = selection;
    return queries.join(', ');
}