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
	<div class="left">{name}</div>

	<div class="right">
		<div class="count">{count}</div>
		<div class="coloredPart" style="background-color: {getCountColor(count)};"></div>
	</div>
</div>

<style>
	.coloredPart {
		width: 8px;
		height: 20px;
		border-radius: 0px 4px 4px 0px;
	}

	.right {
		display: flex;
	}

	.entry {
		font-family: 'Ubuntu';
		flex-grow: 1;
		border: 1px solid var(--border);
		border-radius: 5px;
		display: flex;
		justify-content: space-between;
	}

	.entry:hover {
		background-color: var(--secondary-alt); /* color-mix with current color?*/
	}

	.count {
		padding-left: 8px;
		padding-right: 8px;
	}

	.left {
		padding-right: 4px;
		padding-left: 4px;
	}
</style>
