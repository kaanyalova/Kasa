<script lang="ts">
	let { _value = $bindable(), max_size }: ResolutionInputProps = $props();
</script>

<div class="">
	<div class="resolutionInputContainerInner">
		<label class="resolutionInputLabel" for="resolutionInput">Height</label>
		<input
			type="text"
			class="resolutionInput textInput monoFont"
			id="resolutionInput"
			bind:value={_value}
		/>
	</div>
	{#if _value > max_size}
		<div class="maxSize">
			Max: {max_size}
		</div>
	{/if}
</div>

<style>
	.resolutionInputContainerInner {
		display: flex;
		height: min-content;
	}

	.resolutionInputLabel {
		padding-left: 2px;
		padding-right: 2px;
		background-color: var(--secondary-alt);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.maxSize {
		background-color: var(--secondary-alt);
		padding: 1px;
		padding-left: 2px;
		padding-right: 2px;
	}

	.textInput {
		background-color: var(--background);
		border: 1px solid var(--secondary-alt);
		padding-right: 6px;
		padding-left: 6px;
	}

	.textInput:focus {
		outline: 1px solid var(--accent);
	}
</style>
