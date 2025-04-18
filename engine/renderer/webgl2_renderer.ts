import { EngineGameInstance, EngineGameInstanceUpdates, EngineGameDrawUpdate, DrawUpdateType,
    SPRITE_DATA_SIZE, TERRAIN_CHUNK_STRIDE, TERRAIN_CHUNK_SIZE_BYTES, GUI_VERTEX_SIZE, 
    DEBUG_VERTEX_SIZE} from "../game_interface";
import { EngineAssets, Texture } from "../assets";
import { set_last_error } from "../error";
import { Size, file_extension } from "../helpers";


const TERRAIN_CHUNK_INDEX_COUNT: number = TERRAIN_CHUNK_STRIDE * TERRAIN_CHUNK_STRIDE * 6;

class DrawCommand {
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
        draw.resource0 = vao;
        draw.resource1 = instance_count;
        draw.resource2 = texture;
    }

    static draw_terrain_chunk(
        draw: DrawCommand,
        chunk_x: number,
        chunk_y: number,
        chunk_vao: WebGLVertexArrayObject,
    ) {
        draw.resource0 = chunk_x;
        draw.resource1 = chunk_y;
        draw.resource2 = chunk_vao;
    }
}

class RendererTexture {
    handle: WebGLTexture;
}

class RendererShaders {
    sprites_position_attrloc: number;
    sprites_instance_position_attrloc: number;
    sprites_instance_texcoord_attrloc: number;
    sprites_instance_data_attrloc: number;
    sprites_view_position: WebGLUniformLocation;
    sprites_view_size: WebGLUniformLocation;
    sprites: WebGLProgram;

    proj_sprites_position_attrloc: number;
    proj_sprites_instance_position_attrloc: number;
    proj_sprites_instance_texcoord_attrloc: number;
    proj_sprites_instance_rotation_attrloc: number;
    proj_sprites_view_position: WebGLUniformLocation;
    proj_sprites_view_size: WebGLUniformLocation;
    proj_sprites: WebGLProgram;

    terrain_position_attrloc: number;
    terrain_uv_attrloc: number;
    terrain_view_position: WebGLUniformLocation;
    terrain_view_size: WebGLUniformLocation;
    terrain_chunk_position: WebGLUniformLocation;
    terrain: WebGLProgram;

    gui_position_attrloc: number;
    gui_uv_attrloc: number;
    gui_color_attrloc: number;
    gui_view_size: WebGLUniformLocation;
    gui_font_texture: WebGLUniformLocation;
    gui_image_texture: WebGLUniformLocation;
    gui: WebGLProgram;

    debug_position_attrloc: number;
    debug_color_attrloc: number;
    debug_view_position: WebGLUniformLocation;
    debug_view_size: WebGLUniformLocation;
    debug: WebGLProgram;
}

class TerrainChunkData {
    vao: WebGLVertexArrayObject;
    chunk_buffer: WebGLBuffer;
}

class RendererBuffers {
    sprites_indices: WebGLBuffer;
    sprites_vertex: WebGLBuffer;
    
    sprites_attributes: WebGLBuffer;
    sprites_attributes_len: number = 0;
    sprites_attributes_capacity: number = 0;

    sprites_vao: WebGLVertexArrayObject[] = [];
    sprite_vao_len: number = 0;

    terrain_indices: WebGLBuffer;
    terrain_vertex: WebGLBuffer;
    terrain_chunk_data: Map<number, TerrainChunkData> = new Map();

    gui_indices: WebGLBuffer;
    gui_vertex: WebGLBuffer;
    gui_indices_capacity: number = 0;
    gui_vertex_capacity: number = 0;
    gui_indices_len: number = 0;
    gui_vertex_len: number = 0;
    gui_vao: WebGLVertexArrayObject;

    debug_vertex: WebGLBuffer;
    debug_vertex_capacity: number = 0;
    debug_vertex_len: number = 0;
    debug_vao: WebGLVertexArrayObject;
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
    depth: WebGLRenderbuffer;

    assets: EngineAssets;
    textures: RendererTexture[];
    terrain_texture: RendererTexture;
    font_texture: RendererTexture;
    gui_texture: RendererTexture;

    shaders: RendererShaders;
    buffers: RendererBuffers;

    terrain_chunk_draw_count: number;
    terrain_chunk_draw: DrawCommand[];

    sprite_draw_count: number;
    sprite_draw: DrawCommand[];

    projectile_draw_count: number;
    projectile_draw: DrawCommand[];

    view_x: number;
    view_y: number;

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

        this.sprite_draw_count = 0;
        this.sprite_draw = [];

        this.projectile_draw_count = 0;
        this.projectile_draw = [];

        this.terrain_chunk_draw_count = 0;
        this.terrain_chunk_draw = [];

        this.textures = [];

        this.view_x = 0.0;
        this.view_y = 0.0;

        return true;
    }

    init_default_resources(assets: EngineAssets): boolean {
        this.assets = assets;

        if (!this.compile_shaders()) {
            return false;
        }

        if (!this.setup_textures()) {
            return false;
        }

        this.setup_buffers();
        this.setup_uniforms();

        return true;
    }

    canvas_size(): Size {
        return { width: this.canvas.width, height: this.canvas.height };
    }

    handle_resize(): boolean {
        const canvas = this.canvas;
        const dpr = window.devicePixelRatio;
        const display_width  = Math.round(canvas.element.clientWidth * dpr);
        const display_height = Math.round(canvas.element.clientHeight * dpr);
        if (display_width == canvas.width && display_height == canvas.height) {
            return false;
        }
    
        const ctx = this.ctx;
        canvas.element.width = display_width;
        canvas.element.height = display_height;
        canvas.width = display_width;
        canvas.height = display_height;
    
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
    
        ctx.bindRenderbuffer(ctx.RENDERBUFFER, this.color);
        ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.RGBA8, canvas.width, canvas.height); 
        ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.RENDERBUFFER, this.color);

        // ctx.bindRenderbuffer(ctx.RENDERBUFFER, this.depth);
        // ctx.renderbufferStorageMultisample(ctx.RENDERBUFFER, this.get_samples(), ctx.DEPTH24_STENCIL8, canvas.width, canvas.height);
        // ctx.framebufferRenderbuffer(ctx.DRAW_FRAMEBUFFER, ctx.DEPTH_STENCIL_ATTACHMENT, ctx.RENDERBUFFER, this.depth);
        
        ctx.viewport(0, 0, canvas.width, canvas.height);

        // Screen size uniforms
        ctx.useProgram(this.shaders.sprites);
        ctx.uniform2f(this.shaders.sprites_view_size, canvas.width, canvas.height);

        ctx.useProgram(this.shaders.proj_sprites);
        ctx.uniform2f(this.shaders.proj_sprites_view_size, canvas.width, canvas.height);

        ctx.useProgram(this.shaders.terrain);
        ctx.uniform2f(this.shaders.terrain_view_size, canvas.width, canvas.height);

        ctx.useProgram(this.shaders.gui);
        ctx.uniform2f(this.shaders.gui_view_size, canvas.width, canvas.height);

        ctx.useProgram(this.shaders.debug);
        ctx.uniform2f(this.shaders.debug_view_size, canvas.width, canvas.height);

        return true;
    }

    //
    // Updates
    //

    private create_renderer_texture(texture_id: number): RendererTexture {
        const ctx = this.ctx;
        const bitmap = this.assets.textures_by_id[texture_id].bitmap;

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

    private create_sprites_vao(attributes_offset: number): WebGLVertexArrayObject {
        const ctx = this.ctx;
        let location: number;

        let vao = this.buffers.sprites_vao[this.buffers.sprite_vao_len];
        if (!vao) {
            vao = ctx.createVertexArray();
            this.buffers.sprites_vao.push(vao);
        }
        
        ctx.bindVertexArray(vao);

        // Vertex data
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.sprites_indices);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_vertex);

        location = this.shaders.sprites_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 2, ctx.FLOAT, false, 8, 0);

        // Instance Data
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_attributes);

        location = this.shaders.sprites_instance_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 4, ctx.FLOAT, false, 36, attributes_offset);
        ctx.vertexAttribDivisor(location, 1);

        location = this.shaders.sprites_instance_texcoord_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 4, ctx.FLOAT, false, 36, attributes_offset+16);
        ctx.vertexAttribDivisor(location, 1);

        location = this.shaders.sprites_instance_data_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribIPointer(location, 1, ctx.INT, 36, attributes_offset+32);
        ctx.vertexAttribDivisor(location, 1);

        ctx.bindVertexArray(null);

        return vao;
    }

    private create_projectile_sprites_vao(attributes_offset: number): WebGLVertexArrayObject {
        const ctx = this.ctx;
        let location: number;

        let vao = this.buffers.sprites_vao[this.buffers.sprite_vao_len];
        if (!vao) {
            vao = ctx.createVertexArray();
            this.buffers.sprites_vao.push(vao);
        }
        
        ctx.bindVertexArray(vao);

        // Vertex data
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.sprites_indices);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_vertex);

        location = this.shaders.proj_sprites_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 2, ctx.FLOAT, false, 8, 0);

        // Instance Data
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_attributes);

        location = this.shaders.proj_sprites_instance_position_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 4, ctx.FLOAT, false, 36, attributes_offset);
        ctx.vertexAttribDivisor(location, 1);

        location = this.shaders.proj_sprites_instance_texcoord_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 4, ctx.FLOAT, false, 36, attributes_offset+16);
        ctx.vertexAttribDivisor(location, 1);

        location = this.shaders.proj_sprites_instance_rotation_attrloc;
        ctx.enableVertexAttribArray(location);
        ctx.vertexAttribPointer(location, 1, ctx.FLOAT, false, 36, attributes_offset+32);
        ctx.vertexAttribDivisor(location, 1);

        ctx.bindVertexArray(null);

        return vao;
    }

    private update_sprites(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) {
        const ctx = this.ctx;

        let texture = this.textures[draw_update.texture_id];
        if (!texture) {
            texture = this.create_renderer_texture(draw_update.texture_id);
        }

        if (this.buffers.sprites_attributes_len + draw_update.instance_count > this.buffers.sprites_attributes_capacity) {
            const overflow = (this.buffers.sprites_attributes_len + draw_update.instance_count) - this.buffers.sprites_attributes_capacity;
            this.realloc_sprites_attributes(overflow);
        }

        const attributes_offset = SPRITE_DATA_SIZE * this.buffers.sprites_attributes_len;
        const vao = this.create_sprites_vao(attributes_offset);
        const buffer_data = updates.get_sprites_data(draw_update.instance_base, draw_update.instance_count);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_attributes);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, attributes_offset, buffer_data);

        this.buffers.sprites_attributes_len += draw_update.instance_count;
        this.buffers.sprite_vao_len += 1;

        const count = this.sprite_draw_count;
        this.sprite_draw_count += 1;

        let draw: DrawCommand = this.sprite_draw[count];
        if (!draw) {
            draw = new DrawCommand();
            this.sprite_draw[count] = draw;
        }

        DrawCommand.draw_sprites(
            draw,
            vao,
            draw_update.instance_count,
            texture.handle
        );
    }

    private update_projectile_sprites(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) {
        const ctx = this.ctx;

        let texture = this.textures[draw_update.texture_id];
        if (!texture) {
            texture = this.create_renderer_texture(draw_update.texture_id);
        }

        if (this.buffers.sprites_attributes_len + draw_update.instance_count > this.buffers.sprites_attributes_capacity) {
            const overflow = (this.buffers.sprites_attributes_len + draw_update.instance_count) - this.buffers.sprites_attributes_capacity;
            this.realloc_sprites_attributes(overflow);
        }

        const attributes_offset = SPRITE_DATA_SIZE * this.buffers.sprites_attributes_len;
        const vao = this.create_projectile_sprites_vao(attributes_offset);
        const buffer_data = updates.get_projectile_sprites_data(draw_update.instance_base, draw_update.instance_count);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_attributes);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, attributes_offset, buffer_data);

        this.buffers.sprites_attributes_len += draw_update.instance_count;
        this.buffers.sprite_vao_len += 1;

        const count = this.projectile_draw_count;
        this.projectile_draw_count += 1;

        let draw: DrawCommand = this.projectile_draw[count];
        if (!draw) {
            draw = new DrawCommand();
            this.projectile_draw[count] = draw;
        }

        DrawCommand.draw_sprites(
            draw,
            vao,
            draw_update.instance_count,
            texture.handle
        );
    }

    private create_terrain_chunk_buffer(): TerrainChunkData {
        const ctx = this.ctx;

        const chunk_data = new TerrainChunkData();
        chunk_data.vao = ctx.createVertexArray();
        chunk_data.chunk_buffer = ctx.createBuffer();

        let position = this.shaders.terrain_position_attrloc;
        let uv = this.shaders.terrain_uv_attrloc;

        ctx.bindVertexArray(chunk_data.vao);

        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.terrain_indices);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.terrain_vertex);
        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, 8, 0);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, chunk_data.chunk_buffer);
        ctx.bufferData(ctx.ARRAY_BUFFER, TERRAIN_CHUNK_SIZE_BYTES, ctx.STATIC_DRAW);
        ctx.enableVertexAttribArray(uv);
        ctx.vertexAttribPointer(uv, 2, ctx.FLOAT, false, 8, 0);

        ctx.bindVertexArray(null);

        return chunk_data;
    }

    private update_terrain(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) { 
        let chunk_data = this.buffers.terrain_chunk_data.get(draw_update.chunk_id);
        if (!chunk_data) {
            chunk_data = this.create_terrain_chunk_buffer();
            this.buffers.terrain_chunk_data.set(draw_update.chunk_id, chunk_data);
        }

        const ctx = this.ctx;
        const terrain_data = updates.get_terrain_data(draw_update.chunk_data_offset);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, chunk_data.chunk_buffer);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, terrain_data);
    }

    private draw_terrain(draw_update: EngineGameDrawUpdate) {
        const chunk_data = this.buffers.terrain_chunk_data.get(draw_update.chunk_id);
        if (!chunk_data) {
            return;
        }

        const count = this.terrain_chunk_draw_count;
        this.terrain_chunk_draw_count += 1;

        let draw: DrawCommand = this.terrain_chunk_draw[count];
        if (!draw) {
            draw = new DrawCommand();
            this.terrain_chunk_draw[count] = draw;
        }

        DrawCommand.draw_terrain_chunk(
            draw,
            draw_update.chunk_x,
            draw_update.chunk_y,
            chunk_data.vao
        );
    }

    private update_gui(updates: EngineGameInstanceUpdates, draw_update: EngineGameDrawUpdate) {
        const ctx = this.ctx;
        const buffers = this.buffers;

        buffers.gui_indices_len = draw_update.gui_indices_count;
        buffers.gui_vertex_len = draw_update.gui_vertex_count;

        if (buffers.gui_indices_len == 0){
            return;
        }

        if (buffers.gui_vertex_len > buffers.gui_vertex_capacity) {
            const capacity = buffers.gui_vertex_len + 500;
            this.setup_gui_vertex(capacity);
            this.setup_gui_vao();
        }

        const gui_indices_data = updates.get_gui_indices_data();
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, buffers.gui_indices);
        ctx.bufferSubData(ctx.ELEMENT_ARRAY_BUFFER, 0, gui_indices_data);

        const gui_vertex_data = updates.get_gui_vertex_data();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, buffers.gui_vertex);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, gui_vertex_data);
    }

    private update_view_offset(draw_update: EngineGameDrawUpdate) {
        const ctx = this.ctx;
        this.view_x = draw_update.view_x;
        this.view_y = draw_update.view_y;

        const x = -this.view_x;
        const y = -this.view_y;
    
        ctx.useProgram(this.shaders.sprites);
        ctx.uniform2f(this.shaders.sprites_view_position, x, y);

        ctx.useProgram(this.shaders.proj_sprites);
        ctx.uniform2f(this.shaders.proj_sprites_view_position, x, y);

        ctx.useProgram(this.shaders.terrain);
        ctx.uniform2f(this.shaders.terrain_view_position, x, y);

        ctx.useProgram(this.shaders.debug);
        ctx.uniform2f(this.shaders.debug_view_position, x, y);
    }

    private update_debug(updates: EngineGameInstanceUpdates) {
        const ctx = this.ctx;
        const buffers = this.buffers;

        buffers.debug_vertex_len = updates.get_debug_vertex_count();
        if (buffers.debug_vertex_len == 0) {
            return;
        }

        if (buffers.debug_vertex_len > buffers.debug_vertex_capacity) {
            const capacity = buffers.debug_vertex_len + 500;
            this.setup_debug_vertex(capacity);
            this.setup_debug_vao();
        }

        const debug_vertex_data = updates.get_debug_vertex_data();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, buffers.debug_vertex);
        ctx.bufferSubData(ctx.ARRAY_BUFFER, 0, debug_vertex_data);
    }

    private clear_drawing() {
        this.sprite_draw_count = 0;
        this.projectile_draw_count = 0;
        this.terrain_chunk_draw_count = 0;
        this.buffers.sprites_attributes_len = 0;
        this.buffers.sprite_vao_len = 0;
        this.buffers.debug_vertex_len = 0;
    }

    update(game: EngineGameInstance) {
        let game_updates = game.updates();

        this.clear_drawing();
        
        const updates_count = game_updates.draw_updates_count();
        for (let i = 0; i < updates_count; i += 1) {
            const draw_update = game_updates.get_draw_update(i);
            switch (draw_update.module) {
                case DrawUpdateType.UpdateTerrainChunk: {
                    this.update_terrain(game_updates, draw_update);
                    break;
                }
                case DrawUpdateType.DrawTerrainChunk: {
                    this.draw_terrain(draw_update);
                    break;
                }
                case DrawUpdateType.DrawSprites: {
                    this.update_sprites(game_updates, draw_update);
                    break;
                }
                case DrawUpdateType.DrawProjectileSprites: {
                    this.update_projectile_sprites(game_updates, draw_update);
                    break;
                }
                case DrawUpdateType.UpdateGui: {
                    this.update_gui(game_updates, draw_update);
                    break;
                }
                case DrawUpdateType.UpdateViewOffset: {
                    this.update_view_offset(draw_update);
                    break;
                }
                case DrawUpdateType.DrawDebugInfo: {
                    this.update_debug(game_updates);
                    break;
                }
                default: {
                    console.log(`Warning: A drawing update with an unknown type ${draw_update.module} was received`);
                }
            }
        }
    }

    //
    // Render
    //

    private render_sprites() {
        const SPRITE_INDEX_COUNT: number = 6;
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.sprites);

        for (let i = 0; i < this.sprite_draw_count; i += 1) {
            const draw = this.sprite_draw[i];
            const vao = draw.resource0 as WebGLVertexArrayObject;
            const instance_count = draw.resource1 as number;
            const texture = draw.resource2 as WebGLTexture;

            ctx.activeTexture(ctx.TEXTURE0);
            ctx.bindTexture(ctx.TEXTURE_2D, texture);
    
            ctx.bindVertexArray(vao);
    
            ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, instance_count);
        }
    }

    private render_projectiles() {
        const SPRITE_INDEX_COUNT: number = 6;
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.proj_sprites);

        for (let i = 0; i < this.projectile_draw_count; i += 1) {
            const draw = this.projectile_draw[i];
            const vao = draw.resource0 as WebGLVertexArrayObject;
            const instance_count = draw.resource1 as number;
            const texture = draw.resource2 as WebGLTexture;

            ctx.activeTexture(ctx.TEXTURE0);
            ctx.bindTexture(ctx.TEXTURE_2D, texture);
    
            ctx.bindVertexArray(vao);
    
            ctx.drawElementsInstanced(ctx.TRIANGLES, SPRITE_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0, instance_count);
        }
    }

    private render_terrain_chunks() {
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.terrain);
        ctx.activeTexture(ctx.TEXTURE0);
        ctx.bindTexture(ctx.TEXTURE_2D, this.terrain_texture.handle);

        for (let i = 0; i < this.terrain_chunk_draw_count; i += 1) {
            const draw = this.terrain_chunk_draw[i];
            const batch_x = draw.resource0 as number;
            const batch_y = draw.resource1 as number;
            const vao = draw.resource2 as WebGLVertexArrayObject;

            ctx.uniform2f(this.shaders.terrain_chunk_position, batch_x, batch_y);
    
            ctx.bindVertexArray(vao);
            ctx.drawElements(ctx.TRIANGLES, TERRAIN_CHUNK_INDEX_COUNT, ctx.UNSIGNED_SHORT, 0);
        }
    }

    private render_gui() {
        const ctx = this.ctx;
        const buffers = this.buffers;
        if (buffers.gui_indices_len == 0) {
            return;
        }

        ctx.useProgram(this.shaders.gui);

        ctx.activeTexture(ctx.TEXTURE0);
        ctx.bindTexture(ctx.TEXTURE_2D, this.font_texture.handle);

        ctx.activeTexture(ctx.TEXTURE1);
        ctx.bindTexture(ctx.TEXTURE_2D, this.gui_texture.handle);

        ctx.bindVertexArray(buffers.gui_vao);
        ctx.drawElements(ctx.TRIANGLES, buffers.gui_indices_len, ctx.UNSIGNED_SHORT, 0);
    }

    private render_debug() {
        const ctx = this.ctx;
        const buffers = this.buffers;
        if (buffers.gui_vertex_len == 0) {
            return;
        }

        ctx.useProgram(this.shaders.debug);
        ctx.bindVertexArray(this.buffers.debug_vao);
        ctx.drawArrays(ctx.TRIANGLES, 0, buffers.debug_vertex_len);
    }

    render() {
        const ctx = this.ctx;
        const canvas = this.canvas;

        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, this.framebuffer);
        ctx.clearBufferfv(ctx.COLOR, 0, [0.0, 0.0, 0.0, 1.0]);

        this.render_terrain_chunks();
        this.render_sprites();
        this.render_projectiles();
        this.render_gui();
        this.render_debug();

        ctx.bindFramebuffer(ctx.READ_FRAMEBUFFER, this.framebuffer);
        ctx.bindFramebuffer(ctx.DRAW_FRAMEBUFFER, null);
        ctx.blitFramebuffer(0, 0, canvas.width, canvas.height, 0, 0, canvas.width, canvas.height, ctx.COLOR_BUFFER_BIT, ctx.LINEAR);
    }

    //
    // Reload
    //

    private reload_texture(name: string, texture: Texture): boolean {
        const ctx = this.ctx;
        const old_texture = this.textures[texture.id];
        ctx.deleteTexture(old_texture.handle);

        const new_texture = this.create_renderer_texture(texture.id);

        // Hardcoded textures
        switch (name) {
            case "terrain": { this.terrain_texture = new_texture; break; }
            case "roboto": { this.font_texture = new_texture; break; }
            case "gui": { this.gui_texture = new_texture; break; }
        }

        return true;
    }

    reload_assets(reload_list: string[]): boolean {
        let ok = true;

        for (let asset of reload_list) {
            const ext = file_extension(asset);
            switch (ext) {
                case "png": {
                    const [texture_name, texture] = this.assets.find_texture_by_path(asset);
                    if (texture_name && texture) {
                        ok &&= this.reload_texture(texture_name, texture);
                    }

                    break;
                }
            }
        }

        return ok;
    }

    //
    // Setup
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

        function is_mobile() {
            let check = false;
            (function(a){if(/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a)||/1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0,4))) check = true;})(navigator.userAgent||navigator.vendor||(window as any).opera);
            return check;
        }

        // Don't use msaa on a mobile device
        if (is_mobile()) {
            max_samples = 1;
        }

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

        shaders.sprites_position_attrloc = ctx.getAttribLocation(sprites_program, "in_position");
        shaders.sprites_instance_position_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_position");
        shaders.sprites_instance_texcoord_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_texcoord");
        shaders.sprites_instance_data_attrloc = ctx.getAttribLocation(sprites_program, "in_instance_data");
        shaders.sprites_view_position = ctx.getUniformLocation(sprites_program, "view_position") as any;
        shaders.sprites_view_size = ctx.getUniformLocation(sprites_program, "view_size") as any;
        shaders.sprites = sprites_program;

        // Projectile sprites
        const proj_sprites_shader_source = assets.shaders.get("projectile_sprites");
        if (!proj_sprites_shader_source) {
            set_last_error("Failed to find projectile sprites shader source in assets");
            return false;
        }

        const proj_sprites_vert = create_shader(ctx, ctx.VERTEX_SHADER, proj_sprites_shader_source.vertex);
        const proj_sprites_frag = create_shader(ctx, ctx.FRAGMENT_SHADER, proj_sprites_shader_source.fragment);
        if (!proj_sprites_vert || !proj_sprites_frag) {
            set_last_error("Failed to create projectile sprites shaders");
            return false;
        }

        const proj_sprites_program = create_program(ctx, proj_sprites_vert, proj_sprites_frag);
        if (!proj_sprites_program) {
            set_last_error("Failed to compile projectile sprites shaders");
            return false;
        }

        shaders.proj_sprites_position_attrloc = ctx.getAttribLocation(proj_sprites_program, "in_position");
        shaders.proj_sprites_instance_position_attrloc = ctx.getAttribLocation(proj_sprites_program, "in_instance_position");
        shaders.proj_sprites_instance_texcoord_attrloc = ctx.getAttribLocation(proj_sprites_program, "in_instance_texcoord");
        shaders.proj_sprites_instance_rotation_attrloc = ctx.getAttribLocation(proj_sprites_program, "in_instance_rotation");
        shaders.proj_sprites_view_position = ctx.getUniformLocation(proj_sprites_program, "view_position") as any;
        shaders.proj_sprites_view_size = ctx.getUniformLocation(proj_sprites_program, "view_size") as any;
        shaders.proj_sprites = proj_sprites_program;

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

        shaders.terrain_position_attrloc = ctx.getAttribLocation(terrain_program, "in_position");
        shaders.terrain_uv_attrloc = ctx.getAttribLocation(terrain_program, "in_uv");
        shaders.terrain_view_position = ctx.getUniformLocation(terrain_program, "view_position") as any;
        shaders.terrain_view_size = ctx.getUniformLocation(terrain_program, "view_size") as any;
        shaders.terrain_chunk_position = ctx.getUniformLocation(terrain_program, "chunk_position") as any;
        shaders.terrain = terrain_program;

        // Gui
        const gui_shader_source = assets.shaders.get("gui");
        if (!gui_shader_source) {
            set_last_error("Failed to find gui shader source in assets");
            return false;
        }

        const gui_vert = create_shader(ctx, ctx.VERTEX_SHADER, gui_shader_source.vertex);
        const gui_frag = create_shader(ctx, ctx.FRAGMENT_SHADER, gui_shader_source.fragment);
        if (!gui_vert || !gui_frag) {
            set_last_error("Failed to create gui shaders");
            return false;
        }

        const gui_program = create_program(ctx, gui_vert, gui_frag);
        if (!gui_program) {
            set_last_error("Failed to compile gui shaders");
            return false;
        }

        shaders.gui_position_attrloc = ctx.getAttribLocation(gui_program, "in_position");
        shaders.gui_uv_attrloc = ctx.getAttribLocation(gui_program, "in_uv");
        shaders.gui_color_attrloc = ctx.getAttribLocation(gui_program, "in_color");
        shaders.gui_view_size = ctx.getUniformLocation(gui_program, "view_size") as any;
        shaders.gui_font_texture = ctx.getUniformLocation(gui_program, "fonts_texture") as any;
        shaders.gui_image_texture = ctx.getUniformLocation(gui_program, "images_texture") as any;
        shaders.gui = gui_program;

        // Debug
        const debug_shader_source = assets.shaders.get("debug");
        if (!debug_shader_source) {
            set_last_error("Failed to find debug shader source in assets");
            return false;
        }

        const debug_vert = create_shader(ctx, ctx.VERTEX_SHADER, debug_shader_source.vertex);
        const debug_frag = create_shader(ctx, ctx.FRAGMENT_SHADER, debug_shader_source.fragment);
        if (!debug_vert || !debug_frag) {
            set_last_error("Failed to create debug shaders");
            return false;
        }

        const debug_program = create_program(ctx, debug_vert, debug_frag);
        if (!debug_program) {
            set_last_error("Failed to compile debug shaders");
            return false;
        }

        shaders.debug_position_attrloc = ctx.getAttribLocation(debug_program, "in_position");
        shaders.debug_color_attrloc = ctx.getAttribLocation(debug_program, "in_color");
        shaders.debug_view_position = ctx.getUniformLocation(debug_program, "view_position") as any;
        shaders.debug_view_size = ctx.getUniformLocation(debug_program, "view_size") as any;
        shaders.debug = debug_program;

        // Cleanup
        ctx.deleteShader(sprites_vert);
        ctx.deleteShader(sprites_frag);
        ctx.deleteShader(proj_sprites_vert);
        ctx.deleteShader(proj_sprites_frag);
        ctx.deleteShader(terrain_vert);
        ctx.deleteShader(terrain_frag);
        ctx.deleteShader(gui_vert);
        ctx.deleteShader(gui_frag);
        ctx.deleteShader(debug_vert);
        ctx.deleteShader(debug_frag);

        return true;
    }

    private setup_textures(): boolean {
        // Preload constant textures used in some of the shader program

        const texture_id = this.assets.textures.get("terrain")?.id;
        if (!texture_id) {
            set_last_error("Failed to load terrain texture")
            return false;
        }

        const font_texture_id = this.assets.fonts.get("roboto")?.texture_id;
        if (!font_texture_id) {
            set_last_error("Failed to load font texture")
            return false;
        }

        const gui_texture_id = this.assets.textures.get("gui")?.id;
        if (!gui_texture_id) {
            set_last_error("Failed to load gui texture")
            return false;
        }

        this.terrain_texture = this.create_renderer_texture(texture_id);
        this.font_texture = this.create_renderer_texture(font_texture_id);
        this.gui_texture = this.create_renderer_texture(gui_texture_id);

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

    private setup_sprites_attributes() {
        const ctx = this.ctx;

        // Base sprites buffer can hold 128 sprites (totalling ~4kb)
        // If the number of sprites exceed that limits, the buffer will be resized with `realloc_sprites_attributes`
        const base_capacity = 128;
        this.buffers.sprites_attributes = ctx.createBuffer();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.sprites_attributes);
        ctx.bufferData(ctx.ARRAY_BUFFER, SPRITE_DATA_SIZE * base_capacity, ctx.DYNAMIC_DRAW);

        this.buffers.sprites_attributes_capacity = base_capacity;
        this.buffers.sprites_attributes_len = 0;

        // Prealloc some vertex array objects
        for (let i = 0; i < 16; i+=1) {
            this.buffers.sprites_vao.push(ctx.createVertexArray());
        }
    }

    private realloc_sprites_attributes(overflow: number) {
        const ctx = this.ctx;

        const new_capacity = this.buffers.sprites_attributes_capacity + overflow + 256;
        const new_buffer = ctx.createBuffer();
        const old_buffer = this.buffers.sprites_attributes;

        ctx.bindBuffer(ctx.ARRAY_BUFFER, new_buffer);
        ctx.bufferData(ctx.ARRAY_BUFFER, SPRITE_DATA_SIZE * new_capacity, ctx.DYNAMIC_DRAW);

        ctx.bindBuffer(ctx.COPY_READ_BUFFER, old_buffer);
        ctx.bindBuffer(ctx.COPY_WRITE_BUFFER, new_buffer);
        ctx.copyBufferSubData(ctx.COPY_READ_BUFFER, ctx.COPY_WRITE_BUFFER, 0, 0, SPRITE_DATA_SIZE * this.buffers.sprites_attributes_capacity);

        ctx.bindBuffer(ctx.COPY_READ_BUFFER, null);
        ctx.bindBuffer(ctx.COPY_WRITE_BUFFER, null);
        ctx.deleteBuffer(old_buffer);

        this.buffers.sprites_attributes = new_buffer;
        this.buffers.sprites_attributes_capacity = new_capacity;
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
        this.buffers.terrain_indices = ctx.createBuffer();
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.terrain_indices);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, indices_data, ctx.STATIC_DRAW);

        this.buffers.terrain_vertex = ctx.createBuffer();
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.terrain_vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, vertex_data, ctx.STATIC_DRAW);
    }

    private setup_gui_vertex(capacity?: number) {
        const ctx = this.ctx;
        const buffers = this.buffers;

        const GUI_VERTEX_CAPACITY = capacity ? capacity : 1000;
        const INDEX_SIZE = 2;

        // Indices
        buffers.gui_indices = ctx.createBuffer();
        buffers.gui_indices_capacity = Math.ceil(GUI_VERTEX_CAPACITY  * 1.5);
        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.gui_indices);
        ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, buffers.gui_indices_capacity * INDEX_SIZE, ctx.STATIC_DRAW);

        // Vertex
        buffers.gui_vertex = ctx.createBuffer();
        buffers.gui_vertex_capacity = GUI_VERTEX_CAPACITY;
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.gui_vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, buffers.gui_vertex_capacity * GUI_VERTEX_SIZE, ctx.STATIC_DRAW);
    }
    
    private setup_gui_vao() {
        const ctx = this.ctx;
        const buffers = this.buffers;

        const position = this.shaders.gui_position_attrloc;
        const uv = this.shaders.gui_uv_attrloc;
        const color = this.shaders.gui_color_attrloc;

        if (!buffers.gui_vao) {
            buffers.gui_vao = ctx.createVertexArray();
        }
        
        ctx.bindVertexArray(buffers.gui_vao);

        ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, this.buffers.gui_indices);
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.gui_vertex);

        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, 20, 0);

        ctx.enableVertexAttribArray(uv);
        ctx.vertexAttribPointer(uv, 2, ctx.FLOAT, false, 20, 8);

        ctx.enableVertexAttribArray(color);
        ctx.vertexAttribPointer(color, 4, ctx.UNSIGNED_BYTE, true, 20, 16);

        ctx.bindVertexArray(null);
    }

    private setup_debug_vertex(capacity?: number) {
        const ctx = this.ctx;
        const buffers = this.buffers;
        const DEBUG_VERTEX_CAPACITY = capacity ? capacity : 500;
        buffers.debug_vertex = ctx.createBuffer();
        buffers.debug_vertex_capacity = DEBUG_VERTEX_CAPACITY;
        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.debug_vertex);
        ctx.bufferData(ctx.ARRAY_BUFFER, buffers.debug_vertex_capacity * DEBUG_VERTEX_SIZE, ctx.STATIC_DRAW);
    }

    private setup_debug_vao() {
        const ctx = this.ctx;
        const buffers = this.buffers;

        const position = this.shaders.debug_position_attrloc;
        const color = this.shaders.debug_color_attrloc;

        if (!buffers.debug_vao) {
            buffers.debug_vao = ctx.createVertexArray();
        }
        
        ctx.bindVertexArray(buffers.debug_vao);

        ctx.bindBuffer(ctx.ARRAY_BUFFER, this.buffers.debug_vertex);

        ctx.enableVertexAttribArray(position);
        ctx.vertexAttribPointer(position, 2, ctx.FLOAT, false, 12, 0);

        ctx.enableVertexAttribArray(color);
        ctx.vertexAttribPointer(color, 4, ctx.UNSIGNED_BYTE, true, 12, 8);

        ctx.bindVertexArray(null);
    }

    private setup_buffers() {
        this.setup_sprites_vertex();
        this.setup_sprites_attributes();
        this.setup_terrain_chunk_vertex();
        this.setup_gui_vertex();
        this.setup_gui_vao();
        this.setup_debug_vertex();
        this.setup_debug_vao();
    }

    private setup_uniforms() {
        const ctx = this.ctx;

        ctx.useProgram(this.shaders.sprites);
        ctx.uniform2f(this.shaders.sprites_view_position, 0.0, 0.0);
        ctx.uniform2f(this.shaders.sprites_view_size, this.canvas.width, this.canvas.height);

        ctx.useProgram(this.shaders.proj_sprites);
        ctx.uniform2f(this.shaders.proj_sprites_view_position, 0.0, 0.0);
        ctx.uniform2f(this.shaders.proj_sprites_view_size, this.canvas.width, this.canvas.height);

        ctx.useProgram(this.shaders.terrain);
        ctx.uniform2f(this.shaders.terrain_view_position, 0.0, 0.0);
        ctx.uniform2f(this.shaders.terrain_view_size, this.canvas.width, this.canvas.height);

        ctx.useProgram(this.shaders.gui);
        ctx.uniform2f(this.shaders.gui_view_size, this.canvas.width, this.canvas.height);
        ctx.uniform1i(this.shaders.gui_font_texture, 0);
        ctx.uniform1i(this.shaders.gui_image_texture, 1);

        ctx.useProgram(this.shaders.debug);
        ctx.uniform2f(this.shaders.debug_view_position, 0.0, 0.0);
        ctx.uniform2f(this.shaders.debug_view_size, this.canvas.width, this.canvas.height);
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
