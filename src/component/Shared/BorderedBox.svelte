<script lang="ts">
	let { children }: any = $props();
</script>

<div class="borderedBox">
	{@render children()}
</div>

<style>
	.borderedBox {
		border: 1px solid var(--secondary-alt);
		padding: 4px;
		font-size: smaller;
	}
</style>
