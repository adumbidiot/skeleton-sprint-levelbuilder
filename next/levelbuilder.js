class LevelBuilder{
	
}

LevelBuilder.Renderer = class Renderer{
	constructor(canvasID){
		//Assume a 1920 by 1080, resize later
		//first, accept a cnvas id
		this.canvas = document.getElementById(canvasID);
		//TODO: Resize to own options
		//
	}
}

LevelBuilder.Renderer.UIObject = class UIObject extends Image{
	constructor(){
		
	}
}