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
	<div class=""></div>
	<div class="count">
		<div style="color: #ffffff" class="insides">
			<div style="background-color: {getCountColor(count)};" class="tagCount">
				{formatCount(count)}
			</div>
			<div class="tagName">
				{name}
			</div>
		</div>
	</div>
</div>

<style>
	.insides {
		display: flex;
		flex-grow: 1;
		border-radius: 5px;
	}

	.tagCount {
		min-width: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--background);
		border-radius: 5px 0px 0px 5px;
		color: black;
	}

	.tagName {
		flex-grow: 1;
		padding-left: 5px;
	}

	.entry {
		font-family: 'Ubuntu';
		flex-grow: 1;
		border: 1px solid var(--border);
		border-radius: 5px;
	}

	.entry:hover {
		background-color: var(--secondary-alt);
	}

	.entry.selected {
		background-color: #6b819b;
		border-radius: 5px;
	}
</style>
