window.addEventListener('load', init);

const WASM_FILE = 'sdl2_1.wasm';

let state = {};

async function init() {
	state.canvas = document.querySelector('#viewport');
	state.context = state.canvas.getContext('2d');
	state.canvas.width = 500;
	state.canvas.height = 500;

	state.mod = (await WebAssembly.instantiateStreaming(fetch(WASM_FILE),
		{ env: generateExports() })).instance.exports;
	state.mod.main();

	window.addEventListener('keydown', event => {
		state.mod.key_down_event(convertKeycode(event.code))
	});

	window.addEventListener('keyup', event => {
		state.mod.key_up_event(convertKeycode(event.code))
	});

	let resizeHandler = () => {
		state.canvas.width = window.innerWidth;
		state.canvas.height = window.innerHeight;
	};
	window.addEventListener('resize', resizeHandler);
	resizeHandler();

	run();
}

function convertKeycode(code) {
	return ({
		"KeyA": 0,
		"KeyD": 3,
		"KeyE": 4,
		"KeyQ": 16,
		"KeyS": 18,
		"KeyW": 22,
		"Escape": 100,
	})[code] ?? -1;
}

async function run() {
	while (true) {
		state.mod.tick();

		await new Promise(res => requestAnimationFrame(res));
	}
}

function generateExports() {
	let obj = {};

	obj.console_log = (ptr, len) => {
		let memory = new Uint8Array(state.mod.memory.buffer, ptr, len);
		let str = (new TextDecoder("UTF-8")).decode(memory);
		console.log('%c' + str, 'font-weight: 700; color: #dea584;');
	}

	obj.canvas_set_stroke_color =
		(r, g, b) => state.context.strokeStyle = `rgb(${r},${g},${b})`;
	obj.canvas_stroke = () => state.context.stroke();
	obj.canvas_begin_path = () => state.context.beginPath();
	obj.canvas_move_to = (x, y) => state.context.moveTo(x, y);
	obj.canvas_line_to = (x, y) => state.context.lineTo(x, y);
	obj.canvas_clear = () => {
		let [w, h] = [state.canvas.width, state.canvas.height];
		state.context.clearRect(0, 0, w, h);
		state.context.fillColor = '#000';
		state.context.fillRect(0, 0, w, h);
	};
	obj.canvas_width = () => state.canvas.width;
	obj.canvas_height = () => state.canvas.height;

	obj.sin = Math.sin;
	obj.cos = Math.cos;
	obj.fmod = (num, div) => num % div;

	obj.random = Math.random;

	return obj;
}
