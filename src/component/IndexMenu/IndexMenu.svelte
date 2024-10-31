<script>
	import BorderedBox from '../Shared/BorderedBox.svelte';
	import '../../fonts.css';
	import HorizontalDivider from '../Shared/Dividers/HorizontalDivider.svelte';
	import VerticalDivider from '../Shared/Dividers/VerticalDivider.svelte';

	const path =
		'/home/kaan/Belgeler/0000_Projects/kasa_git_15_10_24/src/component/IndexMenu/IndexMenu.svelte';
</script>

<div class="indexMenu">
	<BorderedBox>
		<li>
			<ul class="sourceEntry">{path}</ul>
			<ul class="sourceEntry">{path}</ul>
			<ul class="sourceEntry">{path}</ul>
			<ul class="sourceEntry">{path}</ul>
		</li>
	</BorderedBox>

	<VerticalDivider width={10}></VerticalDivider>

	<BorderedBox padding={8}>
		<button class="button"> New Folder </button>
		<button class="button"> New Folder </button>

	</BorderedBox>
</div>

<style>
	.sourceEntry {
		color: var(--text);
		padding: 2px;
		border: var(--secondary-alt) solid 1px;
		margin-right: -1px;
		margin-bottom: -1px;
	}
	.title {
		font-size: var(--title);
		color: var(--text);
		font-weight: bold;
	}

	.indexMenu {
		padding: 4px;
		display: flex;
		flex-direction: row;
		height: 100vh;
	}

	.button {
		background-color: var(--accent);
		color: var(--text--opposite);
		padding: 8px;
		padding-left: 30px;
		padding-right: 30px;

		border-radius: 4px;
	}
</style>
