import { set_last_error } from "./error";

export interface Size {
    width: number,
    height: number,
}

export function file_extension(path: string): string {
    const lastDotIndex = path.lastIndexOf('.');
    if (lastDotIndex !== -1) {
        return path.slice(lastDotIndex + 1);
    }
    return '';
}


export async function fetch_text(url: string): Promise<string|null> {
    let response: Response | null = await fetch(url)
        .catch((e) => { set_last_error(`Failed to fetch ${url}`); return null; } );

    if (!response) {
        return null;
    }

    if (!response.ok) {
        set_last_error(`Failed to fetch ${url}`);
        return null;
    }

    return response.text();
}
