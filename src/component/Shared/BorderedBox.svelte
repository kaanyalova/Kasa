<script lang="ts">
	let { children, padding = 4 }: BorderedBoxProps = $props();
</script>

<div class="borderedBox" style="padding: {padding}px;">
	{@render children()}
</div>

<style>
	.borderedBox {
		border: 1px solid var(--secondary-alt);
		padding: 4px;
		width: 100%;
	}
</style>
