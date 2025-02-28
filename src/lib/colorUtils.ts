import Color from 'colorjs.io';

// stolen from
// https://github.com/ppy/osu/blob/790f863e0654fd563b57ab699d6be86895e756ab/osu.Game/Utils/ColourUtils.cs#L10
//
// gradient[n][0] is position
// gradient[n][1] is color
export function sampleFromLinearGradient(gradient: Array<[number, Color]>, point: number): Color {
	if (point < gradient[0][0]) {
		return gradient[0][1];
	}

	for (let i = 0; i < gradient.length - 1; i++) {
		const startStop = gradient[i];
		const endStop = gradient[i + 1];

		if (point >= endStop[0]) {
			continue;
		}

		const startColor = new Color(startStop[1]);
		const endColor = new Color(endStop[1]);

		const range = endStop[0] - startStop[0];
		const positionInRange = point - startStop[0];
		const relative = positionInRange / range;

		//console.log(`${startColor}, ${endColor}`);

		const _gradient = startColor.range(endColor);
		return _gradient(relative);
	}

	return gradient[gradient.length - 1][1];
}

export function formatCount(count: number) {
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
export function getCountColor(count: number): string {
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
