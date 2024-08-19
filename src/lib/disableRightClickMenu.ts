import { info } from "@tauri-apps/plugin-log";

export function disableMenu() {
    // Function to handle the right-click event
    function handleRightClick(event: any) {
        event.preventDefault(); // Prevent the default right-click context menu

        // Check if the right-clicked element or any of its ancestors have the 'clickable' class
        var clickableElement = event.target.closest('.clickable');

        if (clickableElement) {
            info('Right-clicked on a clickable element');
            // Perform any desired action for clickable elements
        } else {

            info('Right-clicking is disabled on this element');
            // Perform any desired action for non-clickable elements
        }
    }

    // Attach the right-click event listener to the document
    document.addEventListener('contextmenu', handleRightClick);







}