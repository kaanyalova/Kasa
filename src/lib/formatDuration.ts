export function formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    let result = `${secs}s`;
    
    if (minutes > 0) {
        result = `${minutes}m ${result}`;
    }

    if (hours > 0) {
        result = `${hours}h ${result}`;
    }
    
    return result;
}

export function formatDurationShort(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    const paddedMinutes = String(minutes).padStart(2, '0');
    const paddedSecs = String(secs).padStart(2, '0');

    let result = `${paddedMinutes}:${paddedSecs}`;

    if (hours > 0) {
        const paddedHours = String(hours).padStart(2, '0');
        result = `${paddedHours}:${result}`;
    }

    return result;
}