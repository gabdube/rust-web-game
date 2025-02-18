import typescript from '@rollup/plugin-typescript';

const plugins = [typescript({
    compilerOptions: {
        target: "ES2020",
    }
})];

export default [
    {
        input: 'engine/engine.ts',
        output: {
            file: 'build/engine.js',
            format: 'es'
        },
        plugins
    },
];
