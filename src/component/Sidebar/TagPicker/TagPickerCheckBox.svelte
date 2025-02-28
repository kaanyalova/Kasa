<script lang="ts">
	let { tagName }: TagPickerCheckBoxProps = $props();
</script>

<div class="tagPickerCheckBox">
	<input
		type="checkbox"
		name=""
		id="tag-{tagName}"
		oncontextmenu={(e) => {
			e.preventDefault();
			return false;
		}}
	/>
</div>

<style>
	input {
		height: 20px;
		width: 20px;
		user-select: none;
	}

	.tagPickerCheckBox {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-right: 8px;
	}
</style>
