import Color from "colorjs.io";

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
            continue
        }


        const startColor = new Color(startStop[1]);
        const endColor = new Color(endStop[1]);




        const range = endStop[0] - startStop[0];
        const positionInRange = point - startStop[0];
        const relative = positionInRange / range;



        console.log(`${startColor}, ${endColor}`);


        const _gradient = startColor.range(endColor);
        return _gradient(relative);

    }



    return gradient[gradient.length - 1][1];
}