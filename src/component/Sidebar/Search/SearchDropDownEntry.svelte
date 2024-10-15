<script lang="ts">
	import { sampleFromLinearGradient } from '$lib/colorUtils';
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

	function formatCount(count: number) {
		//	https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat#scientific_engineering_or_compact_notations
		// no i18n yet
		return new Intl.NumberFormat('en-US', {
			notation: 'compact',
			compactDisplay: 'short'
		}).format(count);
	}

	/**
	 * Returns the color of the box and inside count text
	 * @param count count text
	 */
	function getCountColor(count: number): string {
		// having something like this where tag counts have gradients might be cool
		// https://github.com/ppy/osu/blob/790f863e0654fd563b57ab699d6be86895e756ab/osu.Game/Graphics/OsuColour.cs#L26
		// https://github.com/ppy/osu/blob/790f863e0654fd563b57ab699d6be86895e756ab/osu.Game/Utils/ColourUtils.cs#L10

		const base = Math.log10(count);

		// prettier-ignore
		const gradient: Array<[number,Color]> = [
			// why does ts think that the type of the array is (number | Color)[][]
			[1.0,  new Color('#aaaaaa')], // why is there two of them
			[1.3,  new Color('#4290fb')],
			[1.5,  new Color('#4fc0ff')],
			[1.7,  new Color('#4fffd5')],
			[2.0,  new Color('#7cff4f')],
			[2.5,  new Color('#f6f05c')],
			[2.7,  new Color('#ff8068')],
			[3.0,  new Color('#ff4e6f')],
			[3.5,  new Color('#c645b8')],
			[4.0,  new Color('#6563de')],
			[4.5,  new Color('#18158e')],
			[5.0,  new Color('#000000')],
		];

		const color = sampleFromLinearGradient(gradient, base);

		return color.toString();
	}

	const countColor = getCountColor(count);
	const backgroundColor = new Color(countColor).mix('#ffffff', 0.9);
	const textColor = getColorString(color as string);
</script>

<div class="entry">
	<div class=""></div>
	<div class="count">
		<div style="color: #ffffff" class="insides">
			<div style="background-color: {countColor};" class="tagCount">
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
		flex-grow: 100;
		border-radius: 5px;
	}

	.tagCount {
		min-width: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--background);
		border-radius: 5px 0px 0px 5px;
	}

	.tagName {
		flex-grow: 100;
		padding-left: 5px;
	}

	.entry {
		font-family: 'Ubuntu';
		flex-grow: 100;
		border: 1px solid var(--border);
	}

	.entry:hover {
		background-color: var(--secondary-alt);
	}

	.entry.selected {
		background-color: #6b819b;
		border-radius: 5px;
	}
</style>
