import type { SvelteComponent } from "svelte"
import type { RenderTag } from "svelte/compiler"

type SidebarButtonProps = {
    text: string,
    children: any, // TODO acutal type
    onClick: Function,
}