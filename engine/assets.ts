import { fetch_text } from "./helpers";

export class Texture {

}

export class Shader {
    vertex: string;
    fragment: string;
}

export class EngineAssets {
    raw_bundle: string = "";
    textures: Texture[] = [];
    json: string[] = [];
    shaders: Shader[] = [];

    async load(): Promise<boolean> {
        let raw_bundle = await fetch_text("/assets/bundle.csv");
        if (!raw_bundle) {
            return false;
        }

        this.raw_bundle = raw_bundle;

        return true
    }

}

