<script lang="ts">
	import SearchOptionButton from './SearchOptionButton.svelte';
	import { SearchStore } from './SearchStore.svelte';
</script>

<div class="searchOptions">
	<SearchOptionButton
		title="Tag Search"
		isSelected={SearchStore.searchType == 'TagSearch'}
		onClick={() => (SearchStore.searchType = 'TagSearch')}
	>
		<span class="icon-[material-symbols--tab-search] w-5.5 h-5.5 mr-2"></span>
		Tags
	</SearchOptionButton>

	<SearchOptionButton
		title="In Text Search"
		isSelected={SearchStore.searchType == 'OcrTextSearch'}
		onClick={() => (SearchStore.searchType = 'OcrTextSearch')}
	>
		<span class="icon-[material-symbols--manage-search-rounded] w-5.5 h-5.5 mr-2"></span>
		Text
	</SearchOptionButton>

	<SearchOptionButton
		title="Natural Language Search"
		isSelected={SearchStore.searchType == 'EmbeddingSearch'}
		onClick={() => (SearchStore.searchType = 'EmbeddingSearch')}
	>
		<span class="icon-[material-symbols--image-search] w-4.5 h-4.5 mr-2"></span>
		Natural
	</SearchOptionButton>
</div>

<style>
	.searchOptions {
		display: flex;
		justify-content: space-evenly;
		margin-left: 32px;
		margin-right: 32px;
		margin-bottom: 4px;
	}
</style>
