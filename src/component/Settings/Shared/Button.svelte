<script lang="ts">
	let { onClick }: ButtonProps = $props();
	import '../../../fonts.css';
</script>

<button class="button" onclick={onClick}><slot></slot></button>

<style>
	.button {
		display: flex;
		justify-content: center;
		align-items: center;
		font-size: small;
		background-color: var(--accent);
		color: black;
		padding: 2px;
		padding-left: 4px;
		padding-right: 4px;
		border-radius: 6px;
		margin: 4px;
		position: relative;
		width: 76px;
		height: 20px;
		align-items: center;
	}
	.button:hover {
		background-color: color-mix(in srgb, var(--accent) 90%, black 10%);
	}
</style>
