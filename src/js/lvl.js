window.lvl = function(name){
	this.name = name;
	this.dark = false;
	this.active = null;
	this.grid = true;
	var self = this;
	
	this.gridTemplate = document.createElement('div');
	this.gridTemplate.style.cssText = 'width: 25px; height: 25px; outline: 1px solid black; float: left;';
	this.gridTemplate.type = 'holder';
	
	this.board = document.createElement('div');
	this.board.id = this.name;
	this.board.style.cssText = 'width: 800px; height: 450px; position: relative';
	
	this.background = document.createElement('img');
	this.background.src = './images/background.png';
	this.background.style.cssText = 'width: 800px; height: 450px; z-index: -1; position: absolute; top: 0px; left: 0px;';
	
	
	
	let loopFunc = function(){
		requestAnimationFrame(loopFunc);
	}
	
	requestAnimationFrame(loopFunc);
	
    this.history = [];
	
	//Stupid js "this" crap
	this.down = function(event){
		self.renderEvent(event);
		event.preventDefault();
	}
	this.over = function(event){
		if(!lvl.mouseDown){
			self.renderShadowEvent(event);
		}else{
			self.renderEvent(event);
		}
	}
	this.click = function(event){
		self.renderEvent(event);
	}
    //Placeholders
	this.ondarkchange = function(){
		
	}
    this.setdarkfail = function(){
    
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
	var target = document.getElementById(this.name + (index + 1));
	if(target.block == blockType || !blockType) return;
	if(target.block == 'mask_circle'){
		this.setDark(true);
		return;
	}

    this.history.push({oldBlock: target.block || 'delete', newBlock: blockType, index: index});
	
	//console.log(target.childNodes[0].type == 'shadowBlock'); //type of undefined
	if(target.block /*|| target.childNodes[0].type == 'shadowBlock'*/){
		this.clearTile(index);
	}
	
	if(blockType == 'delete') return;

	target.block = blockType;

    if(blockType == 'mask_circle') return;
	
	var blockSrc = './images/' + blockType + '.png';
	if(blockType.startsWith('Note:')){
		blockSrc = './images/note.png';
	}

	var block = document.createElement('img');
	block.style.cssText = 'width: 25px; height: 25px;';
	block.src = blockSrc;
	block.type = 'block';
	target.appendChild(block);
	this.shadowIndex = null;
}

window.lvl.prototype.renderShadow = function(index, blockType){
	var target = document.getElementById(this.name + (index + 1));
	if(target.block == blockType || !blockType || target.shadowBlockType == blockType) return;
	if(target.block == 'mask_circle'){
		return; //Shut up ill fix it later
	}

	if(target.block){
		return;
	}

	if(blockType == 'delete'){
		return; //TODO: Delete functionality	
	}
	
	if(this.shadowIndex){
		this.clearTile(this.shadowIndex);
	}

	target.shadowBlockType = blockType;

	var block = document.createElement('img');
	block.style.cssText = 'width: 25px; height: 25px; opacity: 0.5;';
	block.src = './images/' + blockType + '.png';
	block.type = 'shadowBlock';
	target.appendChild(block);
	this.shadowIndex = index; //TODO: Add clear functionality, add mouse move event listening to call method
}

//TODO: Fix
window.lvl.prototype.setDark = function(value){
	this.dark = value;
	this.ondarkchange(value);
       
    var index = this.getEmptyTile();
    if(index != -1){
        this.render(index, 'mask_circle');
        return;
    }
    this.setdarkfail();
}
//Disables grid on board
window.lvl.prototype.disableGrid = function(){
	this.grid = false;
	for(var i = 0; i != (32 * 18); i++){
		document.getElementById(this.name + (i+1)).style.outline = '0px';
	}
}
//Enables grid
window.lvl.prototype.enableGrid = function(){
	this.grid = true;
	for(var i = 0; i != (32 * 18); i++){
		document.getElementById(this.name + (i+1)).style.outline = '1px solid black';
	}
}
//Clears a tile at specified index
window.lvl.prototype.clearTile = function(index){
	var target = document.getElementById(this.name + (index + 1));
	while(target.firstChild){
		target.removeChild(target.firstChild);
	}
	target.block = null;
}
//Clears all tiles on board
window.lvl.prototype.clearAllTiles = function(){
	for(var i = 0; i != (18 * 32); i++){
		this.clearTile(i);
	}
}
//Returns index of tile with no data or -1 if all tiles are filled
window.lvl.prototype.getEmptyTile = function(){
    for(var i = 0; i != (18 * 32); i ++){
        var tile = document.getElementById(this.name + (i + 1));
        if(!tile.block){
            return i;
        }
    }
    return -1;
}
//Handler for a render event
window.lvl.prototype.renderEvent = function(event){
	var target = event.target;
	if(target.type == 'block' || target.type == 'shadowBlock'){
		target = target.parentNode;
	}
	
	var index = Number(target.id.slice(this.name.length)) - 1;

	if(event.button == 0){ //Left Click
		this.render(index, this.active);
	}
	if(event.button == 2){ //Right Click
		this.render(index, "delete");
	}
}

window.lvl.prototype.renderShadowEvent = function(event){
	var target = event.target;
	if(target.type == 'shadowBlock'){
		target = target.parentNode;
	}
	var index = Number(target.id.slice(this.name.length)) - 1;
	//this.renderShadow(index, this.active); //TODO: Fix shadow Rendering
}

lvl.prototype.getLevelBuilderData = function(){
	var array = [];
	for(var i = 0; i != (32 * 18); i++){
		var element = document.getElementById(this.name + (i + 1));
		array.push(element.block || 'null');
	}
	return array;
}

lvl.prototype.export1D = function(){
	return this.getLevelBuilderData().map(window.sks.encodeBlockLBL);
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

//TODO: FIX/remove
lvl.prototype.import1D = function(data){
	var array = data.slice(',');
	this.importArray1D(array);
}
//Imports a 1D Array that has already been decoded
lvl.prototype.importArray1D = function(array){
	this.clearAllTiles();
	for(var i = 0; i!= (32 * 18); i++){
		if(array[i] != 0 && array[i] != 'null'){
			this.render(i, array[i]);
		}
	}
	console.log(array);
}
//Imports Line-by-Line representations of levels
lvl.prototype.importLBL = function(data){
	var array = data.split('\n');
	console.log(array);
	for(var i = 0; i != array.length; i++){
		let decoded = window.sks.decodeBlockLBL(array[i]);
		array[i] = decoded;	
	}
	this.importArray1D(array);
}

//TODO: Rename/Remove
lvl.prototype.import = function(raw){
	console.log(raw);
	let data = window.sks.decodeAS3(raw);
	console.log(data);
	this.importArray1D(data);
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
window.lvl.ctrlDown = false;
window.lvl.zDown = false;
document.onmousedown = function(){
	window.lvl.mouseDown = true;
}
document.onmouseup = function(){
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