import { info } from "@tauri-apps/plugin-log";

export function handleSelect(name: string, searchContents: String): string {

    let lastEntry;
    let tags_len;

    const split = searchContents.split(',');
    lastEntry = split[split.length - 1];
    tags_len = split.length;

    searchContents = searchContents.substring(0, searchContents.lastIndexOf(lastEntry as string))

    info(`le = ${lastEntry} name = ${name}`);


    if (tags_len === 1) {
        return `${name}, `;
    }

    // return if the tag is already typed
    if (lastEntry === name) {
        info("they are the same");
        return searchContents as string;
    }




    return `${searchContents}, ${name}`;



}