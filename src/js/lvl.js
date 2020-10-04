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
            let start = performance.now();
			self.bgCtx.clearRect(0, 0, self.background.width, self.background.height);
			self.levelBuilder.drawImage(self.bgCtx);
			self.levelBuilder.drawGrid(self.bgCtx);
			self.levelBuilder.dirty = false;
            let end = performance.now();
            console.log("Dirty Redraw: ", end - start);
		}
		requestAnimationFrame(loopFunc);
	}
	
	loopFunc();
	
	// Stupid js "this" crap
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
    
    this.board.addEventListener("mousemove", (event) => {
        const blockSize = 1920 / 32;
        
        const rect = this.background.getBoundingClientRect();
        
        const xRaw = event.offsetX;
        const yRaw = event.offsetY;
        
        const scaleX = this.background.width / rect.width;
        const scaleY = this.background.height / rect.height;
        
        const x = xRaw * scaleX;
        const y = yRaw * scaleY;
        const index = Math.round(x / blockSize) + (Math.round(y / blockSize) * 32);
        
        console.log("bg_index", index, event);
    });
}
// Generates a board
window.lvl.prototype.generateBoard = function(){
    this.board.appendChild(this.background);
    
	for(var i = 0; i != (18 * 32); i++){
		var grid = this.gridTemplate.cloneNode();
		grid.id = this.name + (i + 1);
		this.board.appendChild(grid);
		
		// Apparently this is necessary as js mouse events are actual trash and dont trigger fast enough for a simple drawing app
		// grid.addEventListener("mouseover", this.over);
        grid.addEventListener("mousemove", this.over);
		grid.addEventListener("mousedown", this.down);
		grid.addEventListener("click", this.click);
	}
	return this.board;
}
// Renders a specified block at specified index
window.lvl.prototype.render = function(index, blockType){
	//this.levelBuilder.internalDirty = true;
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

//Handler for a render event
window.lvl.prototype.renderEvent = function(event, index){
	var target = event.target;
	if(target.type == 'block'){
		target = target.parentNode;
	}
	
	var index = Number(target.id.slice(this.name.length)) - 1;
	
	//mrcl sim
	
	const rect = this.background.getBoundingClientRect();
	
	let boxSize = 1920 / 32;
	
	const xRaw = event.clientX - rect.left;
	const yRaw = event.clientY - rect.top;
	const scaleX = this.background.width / rect.width;
    const scaleY = this.background.height / rect.height; 
	const x = xRaw * scaleX;
	const y = yRaw * scaleY;
	const index1 = Math.floor(x / boxSize) + (Math.floor(y / boxSize) * 32);
	console.log(index, index1);
	if(index1 != index) {
		let el = target.getBoundingClientRect();
		let elX = el.x;
		let elY = el.y;
		console.log('Error', index, index1, event.type);
	}
    index = index1;
	
	if(event.type == "mousedown"){
		if(event.button == 0){
			this.render(index, this.active);
		}
		
		if(event.button == 2){
			this.render(index, "delete");
		}
	}
	
	if(event.type == "mouseover" || event.type == "mousemove"){
		if(window.lvl.mouseDownRight){
			this.render(index, this.active);
		}
		
		if(window.lvl.mouseDownLeft){
			this.render(index, "delete");
		}
	}
}

lvl.prototype.exportLBL = function(){
	return this.levelBuilder.export('lbl');		
}

lvl.prototype.exportPNG = function(cb){
	return this.levelBuilder.getImage().toDataURL('image/png');
}

lvl.prototype.exportDev = function(num){
	this.levelBuilder.setLevel(num);
	return this.levelBuilder.export('as3');
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