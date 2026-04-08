<script lang="ts">
	let isExpanded = $state(false);
</script>

<div class="header">Video Details</div>
<div class="videoInfo">
	<button class="expandableHeader" onclick={() => (isExpanded = !isExpanded)}>
		{#if isExpanded}
			▲ Collapse
		{:else}
			▼ More Details
		{/if}
	</button>

	<ul class="details" class:multicolorRows={!isExpanded}>
		{#if isExpanded}
			<li class="categoryContainer">
				<div class="categoryHeader">Video 📹</div>
				<ul class="categoryInner multicolorRows">
					<li>detailentry</li>
					<li>detailentry</li>
					<li>detailentry</li>
					<li>detailentry</li>
				</ul>
			</li>

			<li class="categoryContainer">
				<div class="categoryHeader">
					Audio <span class="icon-[material-symbols--audio-file]"></span>
				</div>
				<ul class="categoryInner multicolorRows">
					<li>detailentry</li>
					<li>detailentry</li>
					<li>detailentry</li>
					<li>detailentry</li>
				</ul>
			</li>
		{:else}
			<li>stuff</li>
			<li>stuff</li>
		{/if}
	</ul>
</div>

<style>
	.categoryHeader {
		margin-left: 4px;
		margin-right: 4px;
		font-weight: bold;
	}

	.categoryInner {
		margin: 4px;
		border: 1px solid var(--secondary-alt);
	}

	.videoInfo {
		margin: 4px;
	}

	.details {
		font-size: small;
		border: var(--secondary-alt) 1px solid;
		padding: 2px;
		border-radius: 2px;
	}

	.header {
		padding-left: 4px;
		padding-right: 4px;
	}

	.expandableHeader {
		background-color: var(--accent);
		color: var(--text-opposite);
		padding-left: 4px;
		padding-right: 4px;
		font-size: small;
		display: block;
		text-align: start;
		width: 100%;
		font-weight: bold;
		border: 1px solid var(--accent-border);
		border-bottom: none;
	}

	.multicolorRows > li:nth-child(2n) {
		background-color: var(--secondary-alt);
	}

	.multicolorRows > li {
		padding: 2px;
	}
</style>
