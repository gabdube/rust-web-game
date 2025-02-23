import { EngineGameInstance, EngineGameInstanceUpdates, EngineGameDrawUpdate, GraphicsModule, SPRITE_DATA_SIZE } from "../game_interface";
import { EngineAssets } from "../assets";
import { set_last_error } from "../error";
import { Size } from "../helpers";

const TERRAIN_CHUNK_STRIDE: number = 16;
const TERRAIN_CHUNK_INDEX_COUNT: number = TERRAIN_CHUNK_STRIDE * TERRAIN_CHUNK_STRIDE * 6;

class DrawCommand {
    module: GraphicsModule;

    // Types depends on the module
    // See static methods
    resource0: any;
    resource1: any; 
    resource2: any; 

    static draw_sprites(
        draw: DrawCommand,
        vao: WebGLVertexArrayObject,
        instance_count: number,
        texture: WebGLTexture,
    ) {
        draw.module = GraphicsModule.DrawSprites;
        draw.resource0 = vao;
        draw.resource1 = instance_count;
        draw.resource2 = texture;
    }

    static draw_terrain_chunk(
        draw: DrawCommand,
        chunk_x: number,
        chunk_y: number
    ) {
        draw.module = GraphicsModule.DrawTerrainChunk;
        draw.resource0 = chunk_x;
        draw.resource1 = chunk_y;
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

    draw_terrain_position_attrloc: number;
    draw_terrain_screen_size: WebGLUniformLocation;
    draw_terrain_chunk_position: WebGLUniformLocation;
    draw_terrain: WebGLProgram;
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

    terrain_indices: WebGLBuffer;
    terrain_vertex: WebGLBuffer;
    terrain_vao: WebGLVertexArrayObject;
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
        
        const ctx = this.ctx;
        ctx.disable(ctx.CULL_FACE);
        ctx.enable(ctx.BLEND);
        ctx.blendFuncSeparate(ctx.ONE, ctx.ONE_MINUS_SRC_ALPHA, ctx.ONE, ctx.ONE_MINUS_DST_ALPHA);
        ctx.blendEquationSeparate(ctx.FUNC_ADD, ctx.FUNC_ADD);

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
        this.setup_uniforms();

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

        // Screen size uniforms
        ctx.useProgram(this.shaders.draw_sprites);
        ctx.uniform2f(this.shaders.draw_sprites_screen_size, this.canvas.width, this.canvas.height);

        ctx.useProgram(this.shaders.draw_terrain);
        ctx.uniform2f(this.shaders.draw_terrain_screen_size, this.canvas.width, this.canvas.height);
    }

    //
    // Updates
    //

    // The draw array is never reset, instead we try to reuse as much memory as possible to reduce pressure on the GC. This may be a little overkill...
    private next_draw_command(): DrawCommand {
        let draw: DrawCommand;
        if (this.draw.length > this.draw_count) {
            draw = this.draw[this.draw_count];
        } else {
            draw = new DrawCommand();
            this.draw[this.draw_count] = draw;
        }

        this.draw_count += 1;

        return draw
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

    /// Updates the sprites data and queue a drawing command to render them
    private update_sprites(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) {
        let texture = this.textures[draw_update.texture_id];
        if (!texture) {
            texture = this.create_renderer_texture(draw_update.texture_id);
        }
        
        // Creates a unique buffer for each texture
        let buffer = this.buffers.sprites_data.get(draw_update.texture_id);
        if (!buffer) {
            const capacity = draw_update.instance_count + (64 - (draw_update.instance_count % 64));
            buffer = this.create_sprites_buffer(capacity);
            this.buffers.sprites_data.set(draw_update.texture_id, buffer);
        } else if (buffer.capacity < draw_update.instance_count) {
            console.log("TODO REALLOC");
        }

        const ctx = this.ctx;
        const buffer_data = updates.get_sprites_data(draw_update.instance_base, draw_update.instance_count);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.instance_data);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, buffer_data);

        DrawCommand.draw_sprites(
            this.next_draw_command(),
            buffer.vao,
            draw_update.instance_count,
            texture.handle
        );
    }

    private update_terrain(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) { 
    }

    private draw_terrain(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) {
        DrawCommand.draw_terrain_chunk(
            this.next_draw_command(),
            draw_update.chunk_x,
            draw_update.chunk_y
        );
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
                case GraphicsModule.UpdateTerrainChunk: {
                    this.update_terrain(game_updates, draw_update);
                    break;
                }
                case GraphicsModule.DrawTerrainChunk: {
                    this.draw_terrain(game_updates, draw_update);
                    break;
                }
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

        const vao = draw.resource0 as WebGLVertexArrayObject;
        const instance_count = draw.resource1 as number;
        const texture = draw.resource2 as WebGLTexture;

        ctx.useProgram(this.shaders.draw_sprites);

        ctx.activeTexture(ctx.TEXTURE0);
        ctx.bindTexture(ctx.TEXTURE_2D, texture);

        ctx.bindVertexArray(vao);

        ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, instance_count);
    }

    render_terrain_chunk(draw: DrawCommand) {
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.draw_terrain);
        ctx.uniform2f(this.shaders.draw_terrain_chunk_position, draw.resource0, draw.resource1);
        ctx.bindVertexArray(this.buffers.terrain_vao);
        ctx.drawElements(ctx.TRIANGLES, TERRAIN_CHUNK_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0);
    }

    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;

        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);

        // Rendering
        for (let i = 0; i < this.draw_count; i += 1) {
            const draw = this.draw[i];
            switch (draw.module) {
                case GraphicsModule.DrawTerrainChunk: {
                    this.render_terrain_chunk(draw);
                    break;
                }
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
        const shaders = this.shaders;
        
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

        shaders.draw_sprites_position_attrloc = ctx.getAttribLocation(sprites_program, "in_position");
        shaders.draw_sprites_instance_position_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_position");
        shaders.draw_sprites_instance_texcoord_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_texcoord");
        shaders.draw_sprites_screen_size = ctx.getUniformLocation(sprites_program, "screen_size") as any;
        shaders.draw_sprites = sprites_program;

        // Terrain
        const terrain_shader_source = assets.shaders.get("terrain");
        if (!terrain_shader_source) {
            set_last_error("Failed to find terrain shader source in assets");
            return false;
        }

        const terrain_vert = create_shader(ctx, ctx.VERTEX_SHADER, terrain_shader_source.vertex);
        const terrain_frag = create_shader(ctx, ctx.FRAGMENT_SHADER, terrain_shader_source.fragment);
        if (!terrain_vert || !terrain_frag) {
            set_last_error("Failed to create terrain shaders");
            return false;
        }

        const terrain_program = create_program(ctx, terrain_vert, terrain_frag);
        if (!terrain_program) {
            set_last_error("Failed to compile terrain shaders");
            return false;
        }

        shaders.draw_terrain_position_attrloc = ctx.getAttribLocation(terrain_program, "in_position");
        shaders.draw_terrain_screen_size = ctx.getUniformLocation(terrain_program, "screen_size") as any;
        shaders.draw_terrain_chunk_position = ctx.getUniformLocation(terrain_program, "chunk_position") as any;
        shaders.draw_terrain = terrain_program;

        // Cleanup
        ctx.deleteShader(sprites_vert);
        ctx.deleteShader(sprites_frag);
        ctx.deleteShader(terrain_vert);
        ctx.deleteShader(terrain_frag);

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

    private setup_terrain_chunk_vertex() {
        const ctx = this.ctx;
        
        // Generate the indices and the vertex to render a single terrain chunk (16x16)
        // Each chunk local coordinates is stored in the position. Ex: the cell at position [4, 3] will have
        // the following position data: {
        //  4.0, 3.0, (V0)
        //  5.0, 3.0, (V1)
        //  4.0, 4.0, (V2)
        //  5.0, 4.0, (V3)
        // }

        const cells_per_chunk = TERRAIN_CHUNK_STRIDE * TERRAIN_CHUNK_STRIDE;
        const floats_per_cell = 4 * 2;

        // Generate data
        const indices_data = new Uint16Array(6 * cells_per_chunk);
        const vertex_data = new Float32Array(floats_per_cell * cells_per_chunk);

        for (let y=0; y<TERRAIN_CHUNK_STRIDE; y+=1) {
            for (let x=0; x<TERRAIN_CHUNK_STRIDE; x+=1) {
                let index = ((y*TERRAIN_CHUNK_STRIDE)+x);
                let indices_offset = index * 6;
                let vertex_index = index * 4;

                indices_data[indices_offset+0] = vertex_index;
                indices_data[indices_offset+1] = vertex_index + 1;
                indices_data[indices_offset+2] = vertex_index + 2;
                indices_data[indices_offset+3] = vertex_index + 2;
                indices_data[indices_offset+4] = vertex_index + 3;
                indices_data[indices_offset+5] = vertex_index + 1;

                let vertex_offset = index * floats_per_cell;
                vertex_data[vertex_offset+0] = x;
                vertex_data[vertex_offset+1] = y;

                vertex_data[vertex_offset+2] = x+1.0;
                vertex_data[vertex_offset+3] = y;

                vertex_data[vertex_offset+4] = x;
                vertex_data[vertex_offset+5] = y + 1.0;

                vertex_data[vertex_offset+6] = x + 1.0;
                vertex_data[vertex_offset+7] = y + 1.0;
            }
        }

        // Update buffers
        this.buffers.terrain_vao = ctx.createVertexArray();
        ctx.bindVertexArray(this.buffers.terrain_vao);

        this.buffers.terrain_indices = ctx.createBuffer();
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.terrain_indices);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, indices_data, ctx.STATIC_DRAW);

        this.buffers.terrain_vertex = ctx.createBuffer();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.terrain_vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, vertex_data, ctx.STATIC_DRAW);

        // Attributes setup
        const location = this.shaders.draw_terrain_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 2, ctx.FLOAT, false, 8, 0);

        ctx.bindVertexArray(null);
    }

    private setup_buffers() {
        this.setup_sprites_vertex();
        this.setup_terrain_chunk_vertex();
    }

    private setup_uniforms() {
        const ctx = this.ctx;
        ctx.useProgram(this.shaders.draw_sprites);
        ctx.uniform2f(this.shaders.draw_sprites_screen_size, this.canvas.width, this.canvas.height);

        ctx.useProgram(this.shaders.draw_terrain);
        ctx.uniform2f(this.shaders.draw_terrain_screen_size, this.canvas.width, this.canvas.height);
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
