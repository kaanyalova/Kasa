<script>
	import SearchDropDownEntry from '../../../component/Sidebar/Search/SearchDropDownEntry.svelte';

	const nums = Array.from(Array(5000).keys());
</script>

{#each nums as num}
	<SearchDropDownEntry color="" , name="Some Tag" count={num}></SearchDropDownEntry>
{/each}
