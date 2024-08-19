<script>
	import Heart from '../Vector/Heart.svelte';
	import Image from '../Vector/Image.svelte';
	import Video from '../Vector/Video.svelte';
	import Gamepad from '../Vector/Gamepad.svelte';
	import SidebarButton from './SidebarButton.svelte';
	import Bars from '../Vector/Bars.svelte';
	import './SideBarGlobals.scss';
</script>

<div class="categories sidebarContent">
	<ul>
		<li class="topText">Categories</li>
		<li class="seperator topMargin"></li>
		<li>
			<SidebarButton text="Favorites">
				<Heart height={20} width={20}></Heart>
			</SidebarButton>
		</li>

		<li>
			<SidebarButton text="All Content">
				<Bars height={20} width={20}></Bars>
			</SidebarButton>
		</li>

		<li>
			<SidebarButton text="Images">
				<Image height={20} width={20}></Image>
			</SidebarButton>
		</li>

		<li>
			<SidebarButton text="Videos">
				<Video height={20} width={20}></Video>
			</SidebarButton>
		</li>

		<li>
			<SidebarButton text="Games">
				<Gamepad height={20} width={20}></Gamepad>
			</SidebarButton>
		</li>
	</ul>
</div>

<style>
	li {
		list-style-type: none;
		display: flex;
	}

	.seperator {
		border: solid 1px var(--border-color-lighter);
	}

	.seperator.topMargin {
		margin-bottom: 5px;
	}

	.categories {
		width: var(--sidebar-content-width);
	}

	.topText {
		font-size: small;
		padding-bottom: 5px;
		color: var(--text);
	}
</style>
