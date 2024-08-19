<script lang="ts">
	import type { SidebarButtonProps } from './SidebarButton';

	let { text, children }: SidebarButtonProps = $props();
</script>

<button class="sidebarButton">
	<div class="iconContainer">
		{@render children()}
	</div>

	<div class="innerText">{text}</div>
</button>

<style>
	button {
		margin: 3px;
		padding: 0px;
		background: color-mix(in srgb, var(--secondary) 30%, transparent);
		display: flex;
		flex-direction: row;
		border: 1px solid color-mix(in srgb, var(--secondary) 30%, transparent);
		border-radius: 10px;
		color: var(--text);
		font-size: large;
		width: var(--button-width);
		box-shadow: rgba(143, 143, 143, 0.24) 0px 2px 6px;
	}

	button:hover {
		background-color: var(--primary);
		color: var(--text-opposite);
	}

	button:hover > .iconContainer {
		fill: var(--text-opposite);
	}

	.iconContainer {
		z-index: 1;
		position: relative;
		fill: var(--text);
		left: 10px;
	}
	.innerText {
		align-items: center;
		justify-content: center;
		display: flex;
		flex-grow: 100;
	}

	.iconContainer {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	@container (width > 10px) {
	}
</style>
