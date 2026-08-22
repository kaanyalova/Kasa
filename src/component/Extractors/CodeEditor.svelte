<script lang="ts">
	import { commands } from '$lib/tauri_bindings';
	import { python } from '@codemirror/lang-python';
	import { vscodeDark } from '@uiw/codemirror-theme-vscode';
	import { basicSetup, EditorView } from 'codemirror';
	import { onDestroy, onMount } from 'svelte';
	import { keymap } from '@codemirror/view';
	import { indentWithTab } from '@codemirror/commands';
	let { extractorName, fileExtension }: CodeEditorProps = $props();

	let editor: EditorView;
	let editorContainer: HTMLElement;

	/*
	let theme = EditorView.theme({
		backgroundColor: '#1e1e1e',
		foreground: '#9cdcfe',
		caret: '#c6c6c6',
		selection: '#6199ff2f',
		selectionMatch: '#72a1ff59',
		lineHighlight: '#ffffff0f',
		gutterBackground: '#1e1e1e',
		gutterForeground: '#838383',
		gutterActiveForeground: '#fff',
		fontFamily: 'Menlo, Monaco, Consolas, "Andale Mono", "Ubuntu Mono", "Courier New", monospace'
	});
	*/

	export function getCode(): string {
		return editor.state.doc.toString();
	}

	async function initEditor() {
		editor = new EditorView({
			parent: editorContainer,
			doc: 'Hello',
			extensions: [basicSetup, python(), vscodeDark, keymap.of([indentWithTab])]
		});

		let contents = '';

		if (extractorName) {
			contents = await commands.createOrGetExtractorContents(extractorName, fileExtension);
		}

		editor.dispatch({
			changes: {
				from: 0,
				to: editor.state.doc.length,
				insert: contents
			}
		});
	}

	$effect(async () => {
		extractorName;
		fileExtension;

		editor.destroy();
		await initEditor();
	});

	onMount(async () => {
		await initEditor();
	});

	onDestroy(() => {
		editor.destroy();
	});
</script>

<div class="codeEditor" bind:this={editorContainer}></div>

<style>
	.codeEditor {
		display: flex;
		flex-grow: 1;
	}

	:global(.cm-editor .cm-content) {
		font-family: 'UbuntuMono';
		font-size: 18px;
	}
</style>
