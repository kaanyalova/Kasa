<script lang="ts">
	import '../../colors.css';
	import '../../fonts.css';

	let { progress }: ProgressBarProps = $props();
	let percentage = $derived(((progress || 0) * 100).toFixed(1));
</script>

<div class="container">
	<div class="barsContainer">
		<div class="background"></div>
		<div class="foreground" style="width: {percentage}%;"></div>
	</div>

	<div class="progressText">
		{percentage}%
	</div>
</div>

<style>
	.background {
		background-color: var(--secondary-alt);
		width: 100%;
		height: 10px;
		position: absolute;
		border-radius: 4px;
		border: 1px solid var(--border);
	}

	.foreground {
		position: absolute;
		height: 10px;
		background-color: var(--accent);
		transition: width 0.1s ease-in;
		border-radius: 4px;
	}

	.barsContainer {
		flex-grow: 1;
		position: relative;
		height: 10px;
		display: flex;
		align-items: center;
	}

	.container {
		display: flex;
		width: 100%;
		align-items: center;
	}

	.progressText {
		color: var(--text);
		font-size: medium;
		text-align: center;
		margin-left: 8px;
		margin-right: 4px;
		min-width: 45px;
	}
</style>
