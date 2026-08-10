<script lang="ts">
	import { formatCount, getCountColor } from '$lib/colorUtils';
	import TagPickerCheckBox from './TagPickerCheckBox.svelte';

	let { checkboxState, onCheck, tagName, count, style }: TagPickerEntryProps = $props();
</script>

<div class="tagPickerEntry" {style}>
	<div class="tag">
		<div class="leftSide">
			<TagPickerCheckBox {tagName} checkboxState={checkboxState ?? 'unselected'} {onCheck}
			></TagPickerCheckBox>
			<label for="tag-{tagName}">
				<div class="tagName" id="tag-{tagName}">
					{tagName}
				</div>
			</label>

			<div class="count">
				{formatCount(count)}
			</div>
		</div>

		<div
			class="coloredPart fancyTagEndShape"
			style="background-color: {getCountColor(count)};"
		></div>
	</div>
</div>

<style>
	.leftSide {
		padding-left: 4px;
		display: flex;
		flex-grow: 1;
		border: 1px solid var(--secondary-alt);
		border-right: none;
	}
	.tag {
		display: flex;
		border-left: none;
		position: relative;
	}

	.tagName {
		overflow-wrap: break-word;
		word-break: break-all;
		margin-right: 8px;
		width: 150px;
	}

	.count {
		margin-left: auto;
		min-width: 42px;
		color: var(--text);
		text-align: center;
		border-radius: 0px 2px 2% 0px;
	}

	.coloredPart {
		min-width: 12px;
		position: relative;
		min-height: 26px;
		aspect-ratio: auto;
	}
</style>
