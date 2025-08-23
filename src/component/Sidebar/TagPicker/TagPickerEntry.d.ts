type TagPickerEntryProps = {
	tagName: string;
	count: number;
	onCheck: (state: TagPickerCheckboxState, tagName: string) => void;
	checkboxState: TagPickerCheckboxState;
};
