<script>
	import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
	import Heart from '../Vector/Heart.svelte';
	import SidebarButton from './SidebarButton.svelte';
	import './SideBarGlobals.scss';

	function onClickDownload() {
		const webView = new WebviewWindow('download', {
			title: 'Download',
			url: 'downloader',
			decorations: false
		});
	}
</script>

<div class="operations sidebarContent">
	<ul>
		<li class="moreTopMargin"></li>
		<li class="topText">Operations</li>
		<li class="seperator topMargin"></li>
		<li>
			<SidebarButton text="Manage Sources">
				<Heart height={20} width={20}></Heart>
			</SidebarButton>
		</li>
		<li>
			<SidebarButton text="Convert">
				<Heart height={20} width={20}></Heart>
			</SidebarButton>
		</li>

		<li>
			<SidebarButton text="Download" onClick={onClickDownload}>
				<Heart height={20} width={20}></Heart>
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
