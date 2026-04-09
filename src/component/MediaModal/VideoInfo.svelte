<script lang="ts">
	import { humanFileSize } from '$lib/formatBytes';
	import type { VideoInfoProps } from './VideoInfo';
	import { formatDuration } from '$lib/formatDuration';

	let isExpanded = $state(false);

	let { metadata }: VideoInfoProps = $props();

	function formatCodecName(codec: string, codec_long_name: string | null) {
		if (codec_long_name) {
			return `${codec_long_name} (${codec})`;
		}
		return codec;
	}
</script>

<div class="videoInfo">
	<div class="header">
		Video Details

		<!--
		<span class="w-5 h-5 icon-[material-symbols--movie-info]"></span>
		-->
	</div>

	<div class="inner">
		<ul class="details" class:multicolorRows={!isExpanded}>
			{#if isExpanded}
				{#if metadata.video_meta}
					<li class="categoryContainer">
						<div class="categoryHeader">
							<span class="categoryHeaderIcon icon-[material-symbols--movie]"></span> Video
						</div>
						<ul class="categoryInner multicolorRows">
							<li><strong>Duration:</strong> {formatDuration(metadata.duration)}</li>

							<li>
								<strong>Codec:</strong>
								{formatCodecName(metadata.video_meta.codec, metadata.video_meta.codec_long_name)}
							</li>
							<li>
								<strong>Resolution:</strong>
								{metadata.video_meta.width}x{metadata.video_meta.height}
							</li>
							<li><strong>Aspect Ratio:</strong> {metadata.video_meta.aspect_ratio}</li>
							<li><strong>Bit Rate:</strong> {humanFileSize(metadata.video_meta.bit_rate)}/s</li>
							{#if metadata.video_meta.max_rate}<li>
									<strong>Max Rate:</strong>
									{humanFileSize(metadata.video_meta.max_rate)}/s
								</li>{/if}
							<li><strong>Format:</strong> {metadata.video_meta.format}</li>
							{#if metadata.video_meta.color_space}<li>
									<strong>Color Space:</strong>
									{metadata.video_meta.color_space}
								</li>{/if}
							{#if metadata.video_meta.color_range}<li>
									<strong>Color Range:</strong>
									{metadata.video_meta.color_range}
								</li>{/if}
							{#if metadata.video_meta.color_primaries}<li>
									<strong>Color Primaries:</strong>
									{metadata.video_meta.color_primaries}
								</li>{/if}
							{#if metadata.video_meta.color_transfer_characteristic}<li>
									<strong>Color Transfer:</strong>
									{metadata.video_meta.color_transfer_characteristic}
								</li>{/if}
							{#if metadata.video_meta.chroma_location}<li>
									<strong>Chroma Location:</strong>
									{metadata.video_meta.chroma_location}
								</li>{/if}
							{#if metadata.video_meta.has_b_frames}<li><strong>B-Frames:</strong> Yes</li>{/if}
							{#if metadata.video_meta.references}<li>
									<strong>References:</strong>
									{metadata.video_meta.references}
								</li>{/if}
							{#if metadata.video_meta.intra_dc_precision}<li>
									<strong>Intra DC Precision:</strong>
									{metadata.video_meta.intra_dc_precision}
								</li>{/if}
							{#if metadata.video_meta.delay}<li>
									<strong>Delay:</strong>
									{metadata.video_meta.delay}
								</li>{/if}
						</ul>
					</li>
				{/if}

				{#if metadata.audio_meta}
					<li class="categoryContainer">
						<div class="categoryHeader noGaps">
							<span class="categoryHeaderIcon icon-[material-symbols--music-note]"></span> Audio
						</div>
						<ul class="categoryInner multicolorRows">
							<li>
								<strong>Codec:</strong>
								{formatCodecName(metadata.audio_meta.codec, metadata.audio_meta.codec_long_name)}
							</li>
							<li><strong>Channels:</strong> {metadata.audio_meta.channels}</li>
							<li><strong>Sample Rate:</strong> {metadata.audio_meta.rate} Hz</li>
							<li><strong>Bit Rate:</strong> {humanFileSize(metadata.audio_meta.bit_rate)}/s</li>
							{#if metadata.audio_meta.max_rate}<li>
									<strong>Max Rate:</strong>
									{humanFileSize(metadata.audio_meta.max_rate)}/s
								</li>{/if}
							<li><strong>Format:</strong> {metadata.audio_meta.format}</li>
							{#if metadata.audio_meta.channel_layout}<li>
									<strong>Channel Layout:</strong>
									{metadata.audio_meta.channel_layout}
								</li>{/if}
							{#if metadata.audio_meta.frames}<li>
									<strong>Frames:</strong>
									{metadata.audio_meta.frames}
								</li>{/if}
							{#if metadata.audio_meta.align}<li>
									<strong>Align:</strong>
									{metadata.audio_meta.align}
								</li>{/if}
							{#if metadata.audio_meta.delay}<li>
									<strong>Delay:</strong>
									{metadata.audio_meta.delay}
								</li>{/if}
						</ul>
					</li>
				{/if}
			{:else}
				<li><strong>Duration:</strong> {formatDuration(metadata.duration)}</li>
				<li><strong>Bit Rate:</strong> {humanFileSize(metadata.bit_rate)}/s</li>
				{#if metadata.video_meta}
					<li>
						<strong>Codec:</strong>
						{formatCodecName(metadata.video_meta.codec, metadata.video_meta.codec_long_name)}
					</li>
				{/if}
			{/if}
		</ul>

		<button class="expandableHeader" onclick={() => (isExpanded = !isExpanded)}>
			{#if isExpanded}
				▲ Collapse
			{:else}
				▼ More Details
			{/if}
		</button>
	</div>
</div>

<style>
	.noGaps {
		gap: 0 !important;
	}

	.categoryHeaderIcon {
		width: 18px;
		height: 18px;
	}

	.categoryHeader {
		margin-left: 4px;
		margin-right: 4px;
		font-weight: bold;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.categoryInner {
		margin: 4px;
		border: 1px solid var(--secondary-alt);
	}

	.inner {
		margin: 4px;
	}

	.details {
		font-size: small;
		border: var(--secondary-alt) 1px solid;
		padding: 2px;
		border-radius: 2px 2px 0 0;
	}

	.header {
		padding-left: 4px;
		padding-right: 4px;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.expandableHeader {
		background-color: var(--background);
		color: var(--text);
		padding-left: 4px;
		padding-right: 4px;
		font-size: small;
		display: block;
		text-align: start;
		width: 100%;
		font-weight: bold;
		border: 1px solid var(--secondary-alt);
		border-top: none;
		border-radius: 0 0 2px 2px;
		cursor: pointer;
	}

	.expandableHeader:hover {
		background-color: var(--secondary-alt);
	}

	.multicolorRows > li:nth-child(2n) {
		background-color: var(--secondary-alt);
	}

	.multicolorRows > li {
		padding: 2px;
	}
</style>
