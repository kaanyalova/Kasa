<script>
	import Heart from '../Vector/Heart.svelte';
	import SidebarButton from './SidebarButton.svelte';
	import './SideBarGlobals.scss';
</script>

<!-- 
We want to display the last visited collections or possibly "pinned" ones, 
get the empty space (the element with `filler` class) at the bottom and fill it until there is no space left
The "All collections button should be always visible"

Collections should support custom icons/colors by the user
-->
<div class="collections sidebarContent">
	<ul>
		<li class="moreTopMargin"></li>
		<li class="topText">Collections</li>
		<li class="seperator topMargin"></li>
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

	.collections {
		width: var(--sidebar-content-width);
	}
</style>
