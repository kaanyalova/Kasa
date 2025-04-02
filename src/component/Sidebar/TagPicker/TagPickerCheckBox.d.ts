type TagPickerCheckBoxProps = {
	tagName: string;
	onCheck: (state: TagPickerCheckboxState, tagName: string) => void;
	state: TagPickerCheckboxState;
};
type TagPickerCheckboxState = 'selected' | 'unselected' | 'exclude';
