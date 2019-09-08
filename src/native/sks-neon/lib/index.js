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
			this.dirty = true;
	}
		
	isDirty(){
		return this.dirty;
	}
	
	getImage(){
		this.internal.getImage();
	}
	
	// Canvas MUST be 1920 x 1080
	drawImage(ctx){
		let binary = new Uint8ClampedArray(this.internal.getImage());
		let imageData = new ImageData(binary, 1920, 1080);
		ctx.putImageData(imageData, 0, 0);
		
		//for(var i = 0; i != 10; i++
		
		this.dirty = false;
	}
}