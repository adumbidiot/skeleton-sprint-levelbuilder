class LevelBuilder {
	constructor(canvas, assetListURL, toolbar){
		canvas = document.getElementById(canvas);
		toolbar = document.getElementById(toolbar);
		
		this.assets = {};	
		
		LevelBuilder.utils.get(assetListURL).catch(function(err){
			throw err;
		}).then((res) => {
			let list = JSON.parse(res);
			this.renderer = new LevelBuilder.Renderer(canvas, this.assets);
			
			return this.loadAssets(list);
		}).catch((err) => {
			throw err;
		}).then(() => {
			this.renderer.loadComplete();
			this.logic = new LevelBuilder.Logic(this.renderer);
			this.UI = new LevelBuilder.UI(this.logic, this.assets, toolbar, this.renderer.canvas);
		});
	}
	async loadAssets(list){
		for(let i = 0; i != list.length; i++){
			this.assets[list[i].name] = await LevelBuilder.utils.getImage(list[i].path); //Populate as loaded
		}
	}
}

LevelBuilder.Renderer = class Renderer {
	constructor(canvas, assets){
		this.canvas = canvas;
		this.ctx = this.canvas.getContext('2d');
		this.assets = assets;
		this.paused = false;
		this.levelData = new Array(32 * 18);
		this.levelData.fill('00');
		this.loadingLoop = setInterval(this.loadingLoop.bind(this), 1000/60);
	}
	loadingLoop(){
		if(this.assets.logo){
			this.ctx.drawImage(this.assets.logo, 0, 0)
		}
	}
	loadComplete(){
		clearInterval(this.loadingLoop);
		this.loop = setInterval(this.mainLoop.bind(this), 1000/60);
	}
	mainLoop(){
		this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
		this.ctx.drawImage(this.assets.cobble_bg, 0, 0, 800 * 2, 450 * 2);
		for(let i = 0; i != this.levelData.length; i++){
			this.render(this.levelData[i], i);
		}
	}
	render(tile, index){
		let x = (index % 32) * 25 * 2;
		let y = ((index / 32) >> 0) * 25 * 2;
		switch(tile){
			case "00": {
				break;
			}
			case "B0": {
				this.ctx.drawImage(this.assets.block, x, y, 2 * 25, 2 * 25);
				break;
			}
			default: {
				//Note implementation
				break;
			}
		}
	}
	write(data, index){
		this.levelData[index] = data;
	}
	writeArray(array){
		for(let i = 0; i != 32 * 18; i++){
			this.write(array[i], i);
		}
	}
	pause(){
		clearInterval(this.loop);
		this.paused = true;
	}
	resume(){
		this.loop = setInterval(this.mainLoop.bind(this), 1000/60);
		this.paused = false;
	}
}
LevelBuilder.Logic = class Logic {
	constructor(renderer){
		this.renderer = renderer;
		this.state = new Array(18 * 32);
		this.state.fill('00');
		this.history = [];
		this.activeBlock = '';
	}
}

LevelBuilder.UI = class UI {
	constructor(logic, assets, toolbarParent, canvas){
		this.logic = logic;
		this.toolbar = document.createElement('div');
		toolbarParent.appendChild(this.toolbar);
		this.toolbar.style.cssText = 'width: 160px; height: 450px; border: 2px solid black; border-radius: 20px; background-color: #3f3f3f; text-align: center; display:flex; flex-flow:column wrap;';
		this.blocks = {};
		this.active = '';
		
		
		let assetKeys = Object.keys(assets);
		console.log(assetKeys);
		for(let i = 0; i != assetKeys.length; i++){
			if(assetKeys[i] === 'logo'){
				continue;
			}
			
			let block = assets[assetKeys[i]].cloneNode();
			block.type = assetKeys[i];
			block.style.cssText = "width: 45px; height: 45px; border-radius: 10px; margin-top: 4px; margin-left: 4px; margin-right: 4px; box-sizing: border-box;";
			block.onclick = () => {
				if(this.active === block.type){
					this.active = '';
					block.style.border = '';
					return;
				}
				this.setActive(block.type);
			}
			block.ondragstart = (event) => {
				event.preventDefault();
				if(this.active === block.type){
					this.active = '';
					block.style.border = '';
					return;
				}
				this.setActive(block.type);
			}
			this.blocks[assetKeys[i]] = block;
			this.toolbar.appendChild(block);
		}
		
	}
	setActive(block){
		if(this.blocks[this.active]){
			this.blocks[this.active].style.border = '';
		}
		this.active = block;
		this.blocks[block].style.border = '1px dashed white';
		//Communicate w/ logic layer..
		//this.logic.setActive(block);
	}
}

//UTIL
LevelBuilder.utils = {};
LevelBuilder.utils.get = function(path){
	return new Promise(function(resolve, reject){
		let request = new XMLHttpRequest();
		request.open('GET', path);
		request.onloadend = function(){
			if(request.status === 200){
				resolve(request.responseText);
			}else{
				reject(status);
			}
		}
		request.send();
	});
}
LevelBuilder.utils.getImage = function(path){
	return new Promise(function(resolve, reject){
		let img = new Image();
		img.src= path;
		img.onload = function(){
			resolve(img);
		}
		img.onerror = function(){
			reject();
		}
	});
}