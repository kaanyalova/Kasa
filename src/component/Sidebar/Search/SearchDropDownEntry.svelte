<script lang="ts">
	import { formatCount, getCountColor, sampleFromLinearGradient } from '$lib/colorUtils';
	import Color from 'colorjs.io';
	import '../../../fonts.css';

	let { color, count, name }: SearchDropDownEntryProps = $props();

	function getColorString(color: string) {
		if (color !== '') {
			return color;
		} else {
			//var style = getComputedStyle(document.body);
			//const color = style.getPropertyValue('--text');
			const color = '#000000';

			return color;
		}
	}

	const countColor = getCountColor(count);
	const backgroundColor = new Color(countColor).mix('#ffffff', 0.9);
	const textColor = getColorString(color as string);
</script>

<div class="entry">
	<div class="leftSide">
		<div class="name">{name}</div>
		<div class="count">{count}</div>
	</div>

	<div class="coloredPart fancyTagEndShape" style="background-color: {getCountColor(count)};"></div>
</div>

<style>
	.coloredPart {
		width: 8px;
		align-self: stretch;
		height: 20px;
	}

	.leftSide {
		display: flex;
		flex-grow: 1;
		border: 1px solid var(--border);
		border-right: none;
		height: 20px;
	}

	.entry {
		font-family: 'Ubuntu';
		flex-grow: 1;
		display: flex;
		justify-content: space-between;
		height: 20px;
	}

	.entry:hover {
		outline: 1px solid var(--primary);
		background-color: var(--secondary-alt);
		border-right: none;
		/*
		 draw the border closer to the overlapping the colored part, 
		 still not perfect though nobody will notice the border slightly being 
		 over the colored part
		*/
		outline-offset: -1px;
	}

	.count {
		padding-left: 8px;
		padding-right: 8px;
		margin-left: auto;
	}

	.name {
		padding-right: 4px;
		padding-left: 4px;
		word-break: break-word;
		overflow-wrap: break-word;
	}
</style>
