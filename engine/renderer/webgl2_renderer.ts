import { EngineGameInstance, EngineGameInstanceUpdates, EngineGameDrawUpdate, GraphicsModule, SPRITE_DATA_SIZE } from "../game_interface";
import { EngineAssets } from "../assets";
import { set_last_error } from "../error";
import { Size } from "../helpers";

class DrawCommand {
    module: GraphicsModule;
    vao: WebGLVertexArrayObject;
    instance_count: number;

    resource0: any; // Types depends on the module. Ex: For the Sprites GraphicsModule, this is a texture.

    constructor(module: GraphicsModule, vao: WebGLVertexArrayObject, instance_count: number, resources: any[]) {
        this.module = module;
        this.vao = vao;
        this.instance_count = instance_count;
        this.resource0 = resources[0];
    }
}

class RendererTexture {
    handle: WebGLTexture;
}

class RendererShaders {
    draw_sprites_position_attrloc: number;
    draw_sprites_instance_position_attrloc: number;
    draw_sprites_instance_texcoord_attrloc: number;
    draw_sprites_screen_size: WebGLUniformLocation;
    draw_sprites: WebGLProgram;
}

class SpriteDataBuffer {
    vao: WebGLVertexArrayObject;
    instance_data: WebGLBuffer;
    capacity: number;
}

class RendererBuffers {
    sprites_indices: WebGLBuffer;
    sprites_vertex: WebGLBuffer;
    sprites_data: Map<number, SpriteDataBuffer> = new Map();
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
    textures: RendererTexture[];

    shaders: RendererShaders;
    buffers: RendererBuffers;

    draw_count: number;
    draw: DrawCommand[];

    init(): boolean {
        if ( !this.setup_canvas() ) { return false };
        if ( !this.setup_context() ) { return false; }
        if ( !this.setup_framebuffer() ) { return false; }

        this.shaders = new RendererShaders();
        this.buffers = new RendererBuffers();

        this.draw_count = 0;
        this.draw = [];
        this.textures = [];

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

    handle_resize() {
        const canvas = this.canvas;
        const dpr = window.devicePixelRatio;
        const display_width  = Math.round(canvas.element.clientWidth * dpr);
        const display_height = Math.round(canvas.element.clientHeight * dpr);
        if (display_width == canvas.width && display_height == canvas.height) {
            return;
        }
    
        const ctx = this.ctx;
        canvas.element.width = display_width;
        canvas.element.height = display_height;
        canvas.width = display_width;
        canvas.height = display_height;
    
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
    
        ctx.bindRenderbuffer(ctx.RENDERBUFFER, this.color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, ctx.getParameter(ctx.MAX_SAMPLES), ctx.RGBA8, canvas.width, canvas.height); 
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, this.color);
        ctx.viewport(0, 0, canvas.width, canvas.height);
    }

    //
    // Updates
    //

    private push_draw_command(draw: DrawCommand) {
        // The draw array is never reset, instead we try to reuse as much memory as possible to reduce pressure on the GC. This may be a little overkill...
        if (this.draw_count == this.draw.length) {
            this.draw.push(draw);
        } else {
            this.draw[this.draw_count] = draw;
        }

        this.draw_count += 1;
    }

    private create_renderer_texture(texture_id: number): RendererTexture {
        const ctx = this.ctx;
        const bitmap = this.assets.textures_by_id[texture_id];
    
        const texture = new RendererTexture();
        texture.handle = ctx.createTexture();
        ctx.bindTexture(ctx.TEXTURE_2D, texture.handle);
        ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_MAG_FILTER, ctx.LINEAR);
        ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR);
        ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE);
        ctx.texParameterf(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE);
        ctx.texStorage2D(ctx.TEXTURE_2D, 1, ctx.RGBA8, bitmap.width, bitmap.height);
        ctx.texSubImage2D(ctx.TEXTURE_2D, 0, 0, 0, bitmap.width, bitmap.height, ctx.RGBA, ctx.UNSIGNED_BYTE, bitmap);

        this.textures[texture_id] = texture;

        return texture;
    }

    private create_sprites_buffer(capacity: number): SpriteDataBuffer {
        const ctx = this.ctx;
        const buffer = new SpriteDataBuffer();
        let location: number;

        buffer.vao = ctx.createVertexArray();
        buffer.instance_data = ctx.createBuffer();
        buffer.capacity = capacity;
        
        ctx.bindVertexArray(buffer.vao);

        // Vertex data
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.sprites_indices);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_vertex);

        location = this.shaders.draw_sprites_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 2, ctx.FLOAT, false, 8, 0);

        // Instance Data
        ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.instance_data);
        ctx.bufferData(ctx.ARRAY_BUFFER, buffer.capacity * SPRITE_DATA_SIZE, ctx.DYNAMIC_DRAW);

        location = this.shaders.draw_sprites_instance_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 4, ctx.FLOAT, false, 32, 0);
        ctx.vertexAttribDivisor(location, 1);

        location = this.shaders.draw_sprites_instance_texcoord_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 4, ctx.FLOAT, false, 32, 16);
        ctx.vertexAttribDivisor(location, 1);

        ctx.bindVertexArray(null);

        return buffer;
    }

    private update_sprites(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) {
        let buffer = this.buffers.sprites_data.get(draw_update.id);
        if (!buffer) {
            const capacity = draw_update.instance_count + (64 - (draw_update.instance_count % 64));
            buffer = this.create_sprites_buffer(capacity);
        } else if (buffer.capacity < draw_update.instance_count) {
            console.log("TODO REALLOC");
        }

        let texture = this.textures[draw_update.texture_id];
        if (!texture) {
            texture = this.create_renderer_texture(draw_update.texture_id);
        }

        const ctx = this.ctx;
        const buffer_data = updates.get_sprites_data(draw_update.instance_base, draw_update.instance_count);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.instance_data);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, buffer_data);

        this.push_draw_command(new DrawCommand(
            draw_update.module,
            buffer.vao,
            draw_update.instance_count,
            [texture.handle]
        ));
    }

    private clear_drawing() {
        this.draw_count = 0;
    }

    update(game: EngineGameInstance) {
        let game_updates = game.updates();

        this.clear_drawing();
        
        const updates_count = game_updates.draw_updates_count();
        for (let i = 0; i < updates_count; i += 1) {
            const draw_update = game_updates.get_draw_update(i);
            switch (draw_update.module) {
                case GraphicsModule.DrawSprites: {
                    this.update_sprites(game_updates, draw_update);
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
        const SPRITE_INDEX_COUNT: number = 6;
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.draw_sprites);

        ctx.activeTexture(ctx.TEXTURE0);
        ctx.bindTexture(ctx.TEXTURE_2D, draw.resource0);

        ctx.bindVertexArray(draw.vao);

        ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, draw.instance_count);
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

        const shaders = this.shaders;
        shaders.draw_sprites_position_attrloc = ctx.getAttribLocation(sprites_program, "in_position");
        shaders.draw_sprites_instance_position_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_position");
        shaders.draw_sprites_instance_texcoord_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_texcoord");
        shaders.draw_sprites_screen_size = ctx.getUniformLocation(sprites_program, "screen_size") as any,
        shaders.draw_sprites = sprites_program;

        // Cleanup
        ctx.deleteShader(sprites_vert);
        ctx.deleteShader(sprites_frag);

        return true;
    }

    private setup_sprites_vertex() {
        const ctx = this.ctx;

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
    }

    private setup_buffers() {
        this.setup_sprites_vertex();
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
