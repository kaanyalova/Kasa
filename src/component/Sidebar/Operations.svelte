<script>
	import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
	import SidebarButton from './SidebarButton.svelte';
	import './SideBarGlobals.scss';

	function onClickDownload() {
		const webView = new WebviewWindow('download', {
			title: 'Download',
			url: 'downloader',
			decorations: false
		});
	}

	function onClickExtractors() {
		const webView = new WebviewWindow('extractor_code_editor', {
			title: 'Extractors',
			url: 'extractor_code_editor',
			decorations: false,
			maximized: true
		});
	}
</script>

<div class="operations sidebarContent">
	<ul>
		<li class="moreTopMargin"></li>
		<li class="topText">Operations</li>
		<li class="seperator topMargin"></li>

		<li>
			<SidebarButton text="Download" onClick={onClickDownload}>
				<span class="icon-[material-symbols--download] w-6 h-6"></span>

				<!--
				<Heart height={20} width={20}></Heart>
				-->
			</SidebarButton>
		</li>

		<li>
			<SidebarButton text="Extractors" onClick={onClickExtractors}>
				<span class="icon-[material-symbols--label] w-5 h-5"></span>
			</SidebarButton>
		</li>
	</ul>
</div>

<style>
	li {
		display: flex;
	}
	.topText {
		font-size: small;
		padding-bottom: 5px;
		color: var(--text);
	}

	.seperator {
		border: solid 1px var(--border-color-lighter);
	}

	.topMargin {
		margin-bottom: 5px;
	}

	.moreTopMargin {
		margin-bottom: 10px;
	}

	.operations {
		width: var(--sidebar-content-width);
	}
</style>
