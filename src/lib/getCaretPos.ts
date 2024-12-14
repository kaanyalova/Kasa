// AI generated spagetti code, at least it works
export function getCursorPosition(): { top: number; left: number } | null {
	const selection = window.getSelection();

	if (!selection || selection.rangeCount === 0) {
		return null; // No cursor position (e.g., not focused or no selection)
	}

	const range = selection.getRangeAt(0);

	// Create a temporary range to calculate position
	const tempRange = range.cloneRange();
	tempRange.collapse(true); // Collapse to the start of the range (cursor position)

	// Create a temporary span element to get position
	const marker = document.createElement('span');
	marker.style.position = 'absolute';
	marker.style.height = '0';
	marker.style.width = '0';
	marker.style.visibility = 'hidden';

	tempRange.insertNode(marker);

	// Get the marker's position
	const rect = marker.getBoundingClientRect();
	const cursorPosition = {
		top: rect.top + window.scrollY,
		left: rect.left + window.scrollX
	};

	// Clean up the temporary marker
	if (marker.parentNode) {
		marker.parentNode.removeChild(marker);
	}
	console.log(`cursor position: top:${cursorPosition.top}, left:${cursorPosition.left}`);
	return cursorPosition;
}