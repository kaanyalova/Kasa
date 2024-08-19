<script lang="ts">
	import { root } from 'postcss';
	import { sidebarStore } from './SidebarStore.svelte';

	let isMouseDown = $state(false);

	function mouseDown(e: MouseEvent) {
		isMouseDown = true;

		document.documentElement.style.userSelect = 'none';
		document.documentElement.style.webkitUserSelect = 'none'; // workaround for gtkwebkit

		document.addEventListener('mousemove', moveSidebar, false);
		document.addEventListener(
			'mouseup',
			() => {
				document.removeEventListener('mousemove', moveSidebar, false);
			},
			false
		);
	}

	function mouseUp(e: MouseEvent) {
		isMouseDown = false;
		document.documentElement.style.userSelect = 'auto';
		document.documentElement.style.webkitUserSelect = 'auto'; // workaround for gtkwebkit
	}

	function moveSidebar(e: MouseEvent) {
		if (isMouseDown) {
		}

		const size = e.x;
		if (size / 3 < 50) {
			return;
		}

		if (size / 3 > 200) {
			return;
		}

		const root: any = document.querySelector(':root');
		root.style.setProperty('--main-val', size / 3 + 'px');
		sidebarStore.setSize(size / 3);
	}
</script>

<div
	class="parent"
	aria-hidden="true"
	onmousedown={(e) => mouseDown(e)}
	onmouseup={(e) => {
		mouseUp(e);
	}}
>
	<div class="resizeBar">
		<div class="top"></div>
		<div class="bottom"></div>
	</div>

	<div class="invis"></div>
</div>

<style lang="scss">
	.top {
		padding-left: var(--resize-bar-padding);
		height: 32px;
		background-color: var(--border-on-secondary-alt);
	}

	.bottom {
		padding-left: var(--resize-bar-padding);
		background-color: transparent;
		user-select: none;
		background-color: var(--border);
		height: calc(100vh - 32px);
	}

	.parent {
		display: flex;
		flex-direction: row;
		user-select: none;
	}

	.invis {
		position: absolute;
		padding-left: 10px;
		background-color: transparent;
		height: 100vh;
		user-select: none;
	}
</style>
