type TagPickerCheckBoxProps = {
	tagName: string;
	onCheck: (state: TagPickerCheckboxState, tagName: string) => void;
	checkboxState: TagPickerCheckboxState;
};
type TagPickerCheckboxState = 'selected' | 'unselected' | 'exclude';
