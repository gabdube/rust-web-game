import { EngineGameInstance, EngineGameDrawUpdate, GraphicsModule } from "../game_interface";
import { EngineAssets } from "../assets";
import { set_last_error } from "../error";
import { Size } from "../helpers";

const SPRITE_INSTANCE_DATA_SIZE: number = 32;  // Data is shared with the GPU using a mat4x2 for each instance

let once = true;

interface DrawCommand {
    module: GraphicsModule,
    index_count: number,
    instance_count: number,
}

interface RendererShaders {
    draw_sprites_screen_size: WebGLUniformLocation,
    draw_sprites: WebGLProgram,
}

interface RendererBuffers {
    sprites: WebGLVertexArrayObject,
    sprites_indices: WebGLBuffer,
    sprites_vertex: WebGLBuffer,
    sprites_instance_data: WebGLBuffer,
    sprites_instance_data_size: number,
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

    assets: EngineAssets;
    shaders: RendererShaders;
    buffers: RendererBuffers;

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
        this.assets = assets;

        if (!this.compile_shaders()) {
            return false;
        }

        this.setup_buffers();

        return true;
    }

    canvas_size(): Size {
        return { width: this.canvas.width, height: this.canvas.height };
    }

    //
    // Updates
    //

    private push_draw_command(module: GraphicsModule, index_count: number, instance_count: number) {
        // The draw array is never reset, instead we try to reuse as much memory as possible to reduce pressure on the GC. This may be a little overkill...
        const draw = { module, index_count, instance_count }
        if (this.draw_count == this.draw.length) {
            this.draw.push(draw);
        } else {
            this.draw[this.draw_count] = draw;
        }

        this.draw_count += 1;
    }


    private update_sprites(update: EngineGameDrawUpdate) {
        this.push_draw_command(update.module, 0, 0);
    }

    update(game: EngineGameInstance) {
        // Resets the draw count
        this.draw_count = 0;
        
        let game_output = game.updates();
        const updates_count = game_output.draw_updates_count();
        for (let i = 0; i < updates_count; i += 1) {
            const update = game_output.get_draw_update(i);
            switch (update.module) {
                case GraphicsModule.DrawSprites: {
                    this.update_sprites(update);
                    break;
                }
                default: {
                    console.log("Warning: A drawing update with an undefined graphics module was received. This should never happen");
                }
            }
        }
    }

    //
    // Render
    //

    render_sprites(draw: DrawCommand) {
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.draw_sprites);
        ctx.bindVertexArray(this.buffers.sprites);
        ctx.drawElementsInstanced(ctx.TRIANGLES, draw.index_count, ctx.UNSIGNED_SHORT, 0, draw.instance_count);
    }

    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;

        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);

        // Global uniforms
        ctx.useProgram(this.shaders.draw_sprites);
        ctx.uniform2f(this.shaders.draw_sprites_screen_size, this.canvas.width, this.canvas.height);

        // Rendering
        for (let i = 0; i < this.draw_count; i += 1) {
            const draw = this.draw[i];
            switch (draw.module) {
                case GraphicsModule.DrawSprites: {
                    this.render_sprites(draw);
                    break;
                }
                default: {
                    console.log("Warning: A drawing command with an undefined graphic module was received. This should never happen");
                }
            }
        }

        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, this.framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }

    //
    //
    //

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

    private compile_shaders(): boolean {
        const ctx = this.ctx;
        const assets = this.assets;
        
        // Sprites
        const sprites_shader_source = assets.shaders.get("sprites");
        if (!sprites_shader_source) {
            set_last_error("Failed to find sprites shader source in assets");
            return false;
        }

        const sprites_vert = create_shader(ctx, ctx.VERTEX_SHADER, sprites_shader_source.vertex);
        const sprites_frag = create_shader(ctx, ctx.FRAGMENT_SHADER, sprites_shader_source.fragment);
        if (!sprites_vert || !sprites_frag) {
            set_last_error("Failed to create sprites shaders");
            return false;
        }

        const sprites_program = create_program(ctx, sprites_vert, sprites_frag);
        if (!sprites_program) {
            set_last_error("Failed to compile sprites shaders");
            return false;
        }

        this.shaders = {
            draw_sprites_screen_size: ctx.getUniformLocation(sprites_program, "screen_size") as any,
            draw_sprites: sprites_program
        };

        // Cleanup
        ctx.deleteShader(sprites_vert);
        ctx.deleteShader(sprites_frag);

        return true;
    }

    private setup_sprites_buffer() {
        const ctx = this.ctx;

        this.buffers.sprites = ctx.createVertexArray();
        ctx.bindVertexArray(this.buffers.sprites);

        // Indices
        this.buffers.sprites_indices = ctx.createBuffer();
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.sprites_indices);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, new Uint16Array([0, 3, 2, 1, 0, 3]), ctx.STATIC_DRAW);

        // Vertex
        this.buffers.sprites_vertex = ctx.createBuffer();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, new Float32Array([
            0.0, 0.0, // V0
            1.0, 0.0, // V1
            0.0, 1.0, // V2
            1.0, 1.0, // V3
        ]), ctx.STATIC_DRAW);

        const position_index = ctx.getAttribLocation(this.shaders.draw_sprites, "in_position");
        ctx.enableVertexAttribArray(position_index);
        ctx.vertexAttribPointer(position_index, 2, ctx.FLOAT, false, 8, 0);

        // Vertex instance data
        this.buffers.sprites_instance_data_size = 300 * SPRITE_INSTANCE_DATA_SIZE;  // Enough to hold 300 sprites instances (may be reallocated)
        this.buffers.sprites_instance_data = ctx.createBuffer();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_instance_data);
        ctx.bufferData(ctx.ARRAY_BUFFER, this.buffers.sprites_instance_data_size, ctx.DYNAMIC_DRAW);

        const data_position_index = ctx.getAttribLocation(this.shaders.draw_sprites, "in_instance_position");
        ctx.enableVertexAttribArray(data_position_index);
        ctx.vertexAttribPointer(data_position_index, 4, ctx.FLOAT, false, 32, 0);
        ctx.vertexAttribDivisor(data_position_index, 1);

        const data_texcoord_index = ctx.getAttribLocation(this.shaders.draw_sprites, "in_instance_position");
        ctx.enableVertexAttribArray(data_texcoord_index);
        ctx.vertexAttribPointer(data_texcoord_index, 4, ctx.FLOAT, false, 32, 16);
        ctx.vertexAttribDivisor(data_texcoord_index, 1);

        ctx.bindVertexArray(null);
    }

    private setup_buffers() {
        this.buffers = {} as any;
        this.setup_sprites_buffer();
    }
}

function create_shader(ctx: WebGL2RenderingContext, type: GLenum, source: string): WebGLShader|undefined {
    const shader = ctx.createShader(type) as WebGLShader;
    ctx.shaderSource(shader, source);
    ctx.compileShader(shader);
    const success = ctx.getShaderParameter(shader, ctx.COMPILE_STATUS);
    if (success) {
        return shader;
    }

    console.log(ctx.getShaderInfoLog(shader));
    ctx.deleteShader(shader);
}

function create_program(ctx: WebGL2RenderingContext, vertexShader: WebGLShader, fragmentShader: WebGLShader): WebGLProgram|undefined {
    const program = ctx.createProgram() as WebGLProgram;
    ctx.attachShader(program, vertexShader);
    ctx.attachShader(program, fragmentShader);
    ctx.linkProgram(program);
    const success = ctx.getProgramParameter(program, ctx.LINK_STATUS);
    if (success) {
        return program;
    }

    console.log(ctx.getProgramInfoLog(program));
    ctx.deleteProgram(program);
}
