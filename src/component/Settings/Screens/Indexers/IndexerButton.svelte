<script lang="ts">
	let { onClick, text, children, backgroundColor, textColor }: IndexerButtonProps = $props();
</script>

<button
	class="button"
	onclick={onClick}
	style="background-color: {backgroundColor}; color: {textColor};"
>
	<div class="buttonInsides">
		<div class="icon">
			{@render children()}
		</div>

		<div class="buttonText">{@render text()}</div>
	</div>
</button>

<style>
	.icon {
		display: flex;
		flex-grow: 0;
		padding-left: 8px;
		align-items: center;
	}

	.buttonText {
		display: flex;
		justify-content: center;
		/*align-items: center;*/
		flex-grow: 1;
		padding-left: 2px;
		padding-right: 2px;
	}
	.buttonInsides {
		display: flex;
		flex-grow: 1;
	}

	.button {
		display: inline-flex;
		justify-content: center;
		align-items: center;
		background-color: var(--accent);
		color: black;
		padding-top: 4px;
		padding-bottom: 4px;
		border-radius: 6px;

		/*Both right and top have margins in the Settings component*/
		margin-bottom: 6px;
		margin-left: 6px;

		position: relative;
		align-items: center;
		flex-grow: 1;
		font-size: small;
		word-break: break-all;
	}
</style>
