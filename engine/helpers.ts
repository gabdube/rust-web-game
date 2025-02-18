export function file_extension(path: string): string {
    const lastDotIndex = path.lastIndexOf('.');
    if (lastDotIndex !== -1) {
        return path.slice(lastDotIndex + 1);
    }
    return '';
}

