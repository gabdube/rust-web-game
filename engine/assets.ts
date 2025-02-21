import { set_last_error } from "./error";
import { fetch_text, fetch_blob } from "./helpers";


export class Json {
    raw: string;
    value: any;
    constructor(raw: string, value: any) {
        this.raw = raw;
        this.value = value;
    }
}

export class Shader {
    vertex: string;
    fragment: string;
    constructor(vertex: string, fragment: any) {
        this.vertex = vertex;
        this.fragment = fragment;
    }
}

export class EngineAssets {
    raw_bundle: string = "";

    textures: Map<string, ImageBitmap> = new Map();
    textures_by_id: ImageBitmap[] = [];

    json: Map<string, Json> = new Map();
    shaders: Map<string, Shader> = new Map();

    async load(): Promise<boolean> {
        let raw_bundle = await fetch_text("/assets/bundle.csv");
        if (!raw_bundle) {
            return false;
        }

        this.raw_bundle = raw_bundle;
        
        let bundle_loaded = await this.load_bundle();
        if (!bundle_loaded) {
            return false
        }

        return true
    }

    private async load_bundle(): Promise<boolean> {
        let split_line = "\n";
        if (this.raw_bundle.indexOf("\r\n") != -1) {
            split_line = "\r\n";
        }

        const lines = this.raw_bundle.split(split_line);
        let asset_loading_promises: Promise<boolean>[] = [];
        for (let line of lines) {
            if (line.length == 0) {
                continue;
            }

            const args = line.split(";");
            let texture_id = 0;
            
            switch (args[0]) {
                case "TEXTURE": {
                    const name = args[1];
                    const path = args[2];
                    asset_loading_promises.push(this.load_texture(texture_id, name, path));
                    break;
                }
                case "JSON": {
                    const name = args[1];
                    const path = args[2];
                    asset_loading_promises.push(this.load_json(name, path));
                    break;
                }
                case "SHADER": {
                    const name = args[1];
                    const vertex_path = args[2];
                    const fragment_path = args[3];
                    asset_loading_promises.push(this.load_shader(name, vertex_path, fragment_path));
                    break;
                }
                default: {
                    console.log(`Warning: Unknown asset type ${args[0]} in bundle`);
                }
            }
        }

        const results = await Promise.all(asset_loading_promises);
        return results.indexOf(false) == -1;
    }

    private async load_texture(texture_id: number, name: string, path: string): Promise<boolean> {
        const texture_blob = await fetch_blob(path);
        if (!texture_blob) {
            return false;
        }

        const bitmap = await createImageBitmap(texture_blob)
            .catch((_) => { set_last_error(`Failed to decode image ${path}`); return null; } );
        
        if (!bitmap) {
            return false;
        }

        this.textures.set(name, bitmap);
        this.textures_by_id[texture_id] = bitmap;

        return true;
    }

    private async load_json(name: string, path: string): Promise<boolean> {
        const json_text = await fetch_text(path);
        if (!json_text) {
            return false;
        }

        let json_value: any;
        try {
            json_value = JSON.parse(json_text);
        } catch (e) {
            set_last_error(`Failed to parse json resource ${name}`, e.toString());
            return false;
        }

        this.json.set(name, new Json(json_text, json_value));

        return true;
    }

    private async load_shader(name: string, vertex_path: string, fragment_path: string): Promise<boolean> {
        const [vertex_text, fragment_text] = await Promise.all([
            fetch_text(vertex_path),
            fetch_text(fragment_path),
        ]);

        if (!vertex_text || !fragment_text) {
            return false;
        }

        this.shaders.set(name, new Shader(vertex_text, fragment_text));

        return true;
    }

}

