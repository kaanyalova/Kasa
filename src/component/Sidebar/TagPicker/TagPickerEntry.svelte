<script lang="ts">
	import { formatCount, getCountColor } from '$lib/colorUtils';
	import TagPickerCheckBox from './TagPickerCheckBox.svelte';

	let { checkboxState, onCheck, tagName, count }: TagPickerEntryProps = $props();
</script>

<div class="tag">
	<TagPickerCheckBox {tagName} checkboxState={checkboxState ?? 'unselected'} {onCheck}
	></TagPickerCheckBox>
	<label for="tag-{tagName}">
		<div class="tagName">
			{tagName}
		</div>
	</label>

	<div class="count">
		{formatCount(count)}
	</div>

	<div class="coloredPart" style="background-color: {getCountColor(count)};"></div>
</div>

<style>
	.tag {
		display: flex;
		border: 1px solid var(--secondary-alt);
		margin: 2px;
		padding-left: 4px;
		border-radius: 0px 10px 10px 0px;
	}

	.tagName {
		overflow-wrap: break-word;
		word-break: break-all;
		margin-right: 8px;
	}

	.count {
		margin-left: auto;
		min-width: 42px;
		color: var(--text);
		text-align: center;
		border-radius: 0px 2px 2% 0px;
	}

	.coloredPart {
		min-width: 8px;
		border-radius: 0px 4px 4px 0px;
		position: relative;
	}
</style>
