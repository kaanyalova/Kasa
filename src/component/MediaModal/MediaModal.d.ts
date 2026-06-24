type MediaModalProps = {
	imageHash: string;
};

type SidebarProps = {
	data: MediaInfo;
	updateTagsTextBoxContents: Function; // What is the type for state?
};

type MetaEntry = {
	name: string;
	value: string;
	isValueMonospaced: boolean;
	isOneLine: boolean;
};

type ImportInfo = {
	importSource: string;
	importLink: ?string;
};

/*
type MediaInfo = {
    meta: Array<MetaEntry>,
    import: ImportInfo,
    paths: Array<string>,
    tags: Array<MediaTag>
    // see schema.rs
    rawTagsField: string,
    hash: string,
    mediaType: string,
    mime: string,
    aspectRatio: number,
    fileName: string,
}
*/
type MediaTag = {
	name: string;
};
