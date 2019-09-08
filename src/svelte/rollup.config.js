import svelte from 'rollup-plugin-svelte';
import resolve from 'rollup-plugin-node-resolve';

export default {
	input: __dirname + '/src/main.js',
	output: {
		file: __dirname + '/public/bundle.js',
		format: 'iife',
		name: 'SksComponents'
	},
	plugins: [
		svelte(),
		resolve(),
	]
}