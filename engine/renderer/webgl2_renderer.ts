import { DemoGame } from "../../build/game/game";
import { EngineAssets } from "../assets";
import { set_last_error } from "../error";
import { Size } from "../helpers";

enum DrawModule {
    DrawSprites
}

interface DrawCommand {
    module: DrawModule,
    index_count: number,
    instance_count: number,
}

class RendererCanvas {
    element: HTMLCanvasElement;
    width: number;
    height: number;

    constructor(element: HTMLCanvasElement) {
        this.element = element;
        this.width = 0;
        this.height = 0;
    }
}

export class WebGL2Backend {
    canvas: RendererCanvas;
    ctx: WebGL2RenderingContext;
    framebuffer: WebGLFramebuffer;
    color: WebGLRenderbuffer;

    shaders: {
        draw_sprites: WebGLProgram,
    };

    draw_count: number;
    draw: DrawCommand[];

    init(): boolean {
        if ( !this.setup_canvas() ) { return false };
        if ( !this.setup_context() ) { return false; }
        if ( !this.setup_framebuffer() ) { return false; }

        this.draw_count = 0;
        this.draw = [];

        return true;
    }

    init_default_resources(assets: EngineAssets): boolean {
        return true;
    }

    canvas_size(): Size {
        return { width: this.canvas.width, height: this.canvas.height };
    }

    update(game: DemoGame) {
    }

    render_sprites(draw: DrawCommand) {
        const ctx = this.ctx;
        ctx.useProgram(this.shaders.draw_sprites);
        ctx.drawElementsInstanced(ctx.TRIANGLES, draw.index_count, ctx.UNSIGNED_SHORT, 0, draw.instance_count);
    }

    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;

        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);

        for (let i = 0; i < this.draw_count; i += 1) {
            const draw = this.draw[i];
            switch (draw.module) {
                case DrawModule.DrawSprites: {
                    this.render_sprites(draw);
                    break;
                }
            }
        }

        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, this.framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }

    private setup_canvas(): boolean {
        const canvas_elem = document.getElementById("app") as HTMLCanvasElement;
        if (!canvas_elem) {
            set_last_error("Canvas element was not found");
            return false;
        }
    
        const dpr = window.devicePixelRatio;
        const display_width  = Math.round(canvas_elem.clientWidth * dpr);
        const display_height = Math.round(canvas_elem.clientHeight * dpr);
        canvas_elem.width = display_width;
        canvas_elem.height = display_height;

        this.canvas = new RendererCanvas(canvas_elem);
        this.canvas.width = display_width;
        this.canvas.height = display_height;

        return true;
    }

    private setup_context(): boolean {
        const canvas = this.canvas;
        const ctx = canvas.element.getContext("webgl2", {
            alpha: true,
            depth: false,
            stencil: false,
            antialias: false,
            premultipliedAlpha: true,
            preserveDrawingBuffer: false,
        });

        if (!ctx) { 
            set_last_error("Webgl2 not supported");
            return false;
        }
    
        this.ctx = ctx;
        this.ctx.viewport(0, 0, canvas.width, canvas.height);

        return true;
    }

    private setup_framebuffer(): boolean {
        const canvas = this.canvas;
        const ctx = this.ctx;
        const framebuffer = ctx.createFramebuffer();
        if (!framebuffer) {
            set_last_error("Failed to create the renderer framebuffer");
            return false;
        }

        const color = ctx.createRenderbuffer();
        if (!color) {
            set_last_error("Failed to create the renderer color render buffer");
            return false;
        }

        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, framebuffer);

        ctx.bindRenderbuffer(ctx.RENDERBUFFER, color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.RGBA8, canvas.width, canvas.height); 
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, color);

        this.framebuffer = framebuffer;
        this.color = color;

        return true;
    }

    private get_samples(): number {
        let max_samples = this.ctx.getParameter(this.ctx.MAX_SAMPLES);

        // We don't need more than 4x msaa
        if (max_samples > 4) {
            max_samples = 4;
        }

        return max_samples
    }
}
