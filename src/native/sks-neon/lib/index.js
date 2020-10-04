var addon = require('../native');

console.log(addon.hello());

let data = ['b0', 'a1'];
console.log(addon.export1DPatch(data));

module.exports.hello = addon.hello;

module.exports.encodeBlockLBL = addon.encodeBlockLBL;

module.exports.decode = addon.decode;

module.exports.LevelBuilder = class LevelBuilder {
	constructor() {
		this.internal = new addon.LevelBuilder();
		this.internalDirty = true; //Temp until i can render from js
		this.internalData = null;
		
		this.grid = true;
		this.dirty = true;
		
	}

	enableGrid() {
		this.grid = true;
		this.dirty = true;
	}

	disableGrid() {
		this.grid = false;
		this.dirty = true;
	}

	isDirty() {
		return this.dirty;
	}

	getImage() {
		this.internal.getImage();
	}
	
	getLevelData() {
		return this.internal.getLevelData();
	}
	
	getImage() {
		let binary = new Uint8ClampedArray(this.internal.getImage());
        // let imageData = new ImageData(binary, 800, 600);
		// let imageData = new ImageData(binary, 1280, 720);
		let imageData = new ImageData(binary, 1920, 1080);
		let canvas = document.createElement('canvas');
		canvas.width = imageData.width;
		canvas.height = imageData.height;
		canvas.getContext('2d').putImageData(imageData, 0, 0);
		return canvas;
	}

	// Canvas MUST be 1920 x 1080
	drawImage(ctx) {
		ctx.clearRect(0, 0, 1920, 1080);
		
		if(this.internalDirty || true) {
			let img = this.getImage();
			ctx.drawImage(img, 0, 0, 1920, 1080);
			this.internalData = img;
			this.internalDirty = false;
		} else {
			ctx.drawImage(this.internalData, 0, 0, 1920, 1080);
		}

		this.dirty = false;
	}

	drawGrid(ctx) {
		if(!this.grid) return;
		let boxSize = 1920 / 32;
		ctx.beginPath();
		ctx.lineWidth = "4";
		ctx.strokeStyle = "black";
		for (var i = 0; i < 32 * 18; i++) {
			let y = (i / 32) | 0;
			let x = i % 32;
			ctx.rect(x * boxSize, y * boxSize, boxSize, boxSize);
		}
		ctx.stroke();
	}
	
	addBlock(i, block){
		this.internal.addBlock(i, block);
		this.dirty = true;
	}
	
	export(type){
		return this.internal.export(type);
	}
	
	exportLevel(){
		return this.internal.exportLevel();
	}
	
	setDark(val){
		this.internal.setDark(val);
	}
	
	getDark(){
		return this.internal.getDark();
	}
	
	import(data){
		this.dirty = true;
		return this.internal.import(data);
	}
	
	setLevel(lvl){
		this.internal.setLevel(lvl);
	}
	
	callThisFunctionToDieInstantly(){
		this.internal.callThisFunctionToDieInstantly();
	}
}
