<script lang="ts">
	import { info } from '@tauri-apps/plugin-log';
	import type { SidebarButtonProps } from './SidebarButton';
	import './SideBarGlobals.scss';

	let { text, children, onClick }: SidebarButtonProps = $props();
</script>

<button
	class="sidebarButton"
	onclick={() => {
		onClick();
	}}
>
	<div class="iconContainer">
		{@render children()}
	</div>

	<div class="innerText">{text}</div>
</button>

<style>
	button {
		margin: 3px;
		padding: 2px;
		background: var(--primary);
		display: flex;
		flex-direction: row;
		flex-grow: 1;
		border: 1px solid var(--secondary-transparent);
		color: var(--text-opposite);
		border-radius: 10px;
		font-size: large;
		width: var(--button-width);
	}

	button:hover {
		background-color: var(--secondary-transparent);
		color: var(--text);
	}

	.innerText {
		align-items: center;
		justify-content: center;
		display: flex;
		flex-grow: 1;
	}

	button:hover > .iconContainer {
		fill: var(--text);
	}

	.iconContainer {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		fill: var(--text-opposite);
		position: relative;
		left: 10px;
	}
</style>
