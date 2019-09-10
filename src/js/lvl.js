window.lvl = function(name){
	this.name = name;
	this.active = null;
	var self = this;
	
	this.gridTemplate = document.createElement('div');
	this.gridTemplate.style.cssText = 'width: 25px; height: 25px; float: left;';
	this.gridTemplate.type = 'holder';
	
	this.board = document.createElement('div');
	this.board.id = this.name;
	this.board.style.cssText = 'width: 800px; height: 450px; position: relative';
	
	this.background = document.createElement('canvas');
	this.background.width = 1920;
	this.background.height = 1080;
	this.background.style.cssText = 'width: 800px; height: 450px; z-index: -1; position: absolute; top: 0px; left: 0px;';
	this.bgCtx = this.background.getContext('2d');
	
	this.levelBuilder = new window.sks.LevelBuilder();
	console.log(this.levelBuilder);
	
	let loopFunc = function(){
		if(self.levelBuilder.isDirty()){
			//TODO: Clear Canvas
			self.levelBuilder.drawImage(self.bgCtx);
			self.levelBuilder.render(self.bgCtx);
			self.levelBuilder.drawGrid(self.bgCtx);
		}
		requestAnimationFrame(loopFunc);
	}
	
	loopFunc();
	
	//Stupid js "this" crap
	this.down = function(event){
		self.renderEvent(event);
		event.preventDefault();
	}
	this.over = function(event){
		self.renderEvent(event);
	}
	this.click = function(event){
		self.renderEvent(event);
	}
}
//Generates a board
window.lvl.prototype.generateBoard = function(){
	for(var i = 0; i != (18 * 32); i++){
		var grid = this.gridTemplate.cloneNode();
		grid.id = this.name + (i + 1);
		this.board.appendChild(grid);
		
		grid.addEventListener("mouseover", this.over);
		grid.addEventListener("mousedown", this.down);
		grid.addEventListener("click", this.click);
	}
	this.board.appendChild(this.background);
	return this.board;
}
//Renders a specified block at specified index
window.lvl.prototype.render = function(index, blockType){
	if (!this.active) return;
	if(blockType == 'delete'){
		blockType = 'null';
	}
	this.levelBuilder.addBlock(index, blockType);	
}

window.lvl.prototype.setDark = function(value){
	this.levelBuilder.setDark(value);
}
//Disables grid on board
window.lvl.prototype.disableGrid = function(){
	this.levelBuilder.disableGrid();
}
//Enables grid
window.lvl.prototype.enableGrid = function(){
	this.levelBuilder.enableGrid();
}
//Clears a tile at specified index
window.lvl.prototype.clearTile = function(index){
	this.levelBuilder.addBlock(index, 'null');	
}
//Clears all tiles on board
window.lvl.prototype.clearAllTiles = function(){
	for(var i = 0; i != (18 * 32); i++){
		this.clearTile(i);
	}
}

//Handler for a render event
window.lvl.prototype.renderEvent = function(event){
	var target = event.target;
	if(target.type == 'block'){
		target = target.parentNode;
	}
	
	var index = Number(target.id.slice(this.name.length)) - 1;
	
	if(event.type == "mousedown"){
		if(event.button == 0){
			this.render(index, this.active);
		}
		
		if(event.button == 2){
			this.render(index, "delete");
		}
	}
	
	if(event.type == "mouseover"){
		if(window.lvl.mouseDownRight){
			this.render(index, this.active);
		}
		
		if(window.lvl.mouseDownLeft){
			this.render(index, "delete");
		}
	}
}

lvl.prototype.export1D = function(){
	return this.levelBuilder.exportLevel().map(window.sks.encodeBlockLBL);
}

lvl.prototype.exportLBL = function(){
	var data = this.export1D();
	var out = '';
	for(var i = 0; i != data.length; i++){
		out += data[i] + '\n';
	}
	return out;			
}

lvl.prototype.exportPNG = function(cb){
	var array = this.export1D();
	var can = document.createElement('canvas');
	can.width = '800';
	can.height = '450';
	
	var context = can.getContext('2d');
	
	var back = new Image();
	back.src= './images/background.png';
	context.drawImage(back, 0, 0, 800, 450);
	
	var count = 0;
	var total = 0;
	for(var i = 0; i != 18; i++){
		for(var j = 0; j != 32; j++){
			var drawing = new Image();
			if(window.sks.decodeBlockLBL(array[( i * 32) + j]) != 'null'){
				count++;
				total++;
				drawing.onload = (function() {
					var a = drawing;
					var x = j;
					var y = i;
					return function(){
  	 					context.drawImage(a, x * 25, y * 25, 25, 25);
						count--;
						if(count == 0){
							var output = can.toDataURL('image/png');
							cb(output);
						}
					}
				})();
				drawing.src = './images/' + window.sks.decodeBlockLBL(array[(i * 32) + j]) + '.png';
			}
		}
	}
	if(total == 0){
		var output = can.toDataURL('image/png');
		cb(output);
	}
}

lvl.prototype.exportDev = function(num){
	num = num || 'x';
	var array = this.export1D();
	return window.sks.encodeAS3(num, array);
}

// Guesses the import format and imports
lvl.prototype.import = function(data){
	let res = this.levelBuilder.import(data);
	document.getElementById('dark').checked = this.levelBuilder.getDark();
	return res;
}

function checkCtrlZ(){
    if(window.lvl.zDown == true && window.lvl.ctrlDown == true && history.length > 0){
         var undo = level.history[level.history.length - 1];
         console.log(undo);
         level.render(undo.index, undo.oldBlock);
         level.history.shift();  
    }
}

window.lvl.mouseDown = false;
window.lvl.mouseDownLeft = false;
window.lvl.mouseDownRight = false;

window.lvl.ctrlDown = false;
window.lvl.zDown = false;
document.onmousedown = function(e){
	if(e.button == 0){
		window.lvl.mouseDownRight = true;
	}else if(e.button == 2){
		window.lvl.mouseDownLeft = true;
	}
	window.lvl.mouseDown = true;
}
document.onmouseup = function(e){
	if(e.button == 0){
		window.lvl.mouseDownRight = false;
	}else if(e.button == 2){
		window.lvl.mouseDownLeft = false;
	}
	window.lvl.mouseDown = false;
}
document.onkeydown = function(event){
    switch(event.keyCode){
        case 17: {
            window.lvl.ctrlDown = true;
            break;
        }
        case 90: {
            window.lvl.zDown = true;
            break;
        }
    }
    checkCtrlZ();
}
document.onkeyup = function(event){
    switch(event.keyCode){
        case 17: {
            window.lvl.ctrlDown = false;
            break;
        }
        case 90: {
            window.lvl.zDown = false;
            break;
        }
    }
    checkCtrlZ();
}