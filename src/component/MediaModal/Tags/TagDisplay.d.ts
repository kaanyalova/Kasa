type TagDisplayProps = {
    isInEditMode: boolean,
    initialEditBoxContents: string,
    updateTagsTextBoxContents: Function,
    data: MediaInfo,
}

type CursorPosition = {
    top: number | null,
    left: number | null,
}