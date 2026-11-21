import type { Snippet } from 'svelte';

type SearchOptionButtonProps = {
	title: string;
	children: Snippet;
	isSelected: boolean;
	onClick: () => void;
};
