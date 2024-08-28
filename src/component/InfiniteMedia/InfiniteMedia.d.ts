type Item = {
    pos_x: number;
    pos_y: number;
    hash: string;
}

type ImageRow = {
    index: number;
    height: number;
    images: Array<ImagePlacement>;
};

type ImagePlacement = {
    x_relative: number;
    y_relative: number;
    width: number;
    height: number;
    hash: string;
};

type ImageProps = {
    width: number;
    height: number;
    offset_x: number;
    offset_y: number;
    hash: string;
    isSelected: boolean;
}